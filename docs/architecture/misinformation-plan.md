# Misinformation Modeling — Architecture Plan

**Status:** Proposed (not implemented).
**Author / Owner:** TBD.
**Related work:** `signal_ignored_penalty` (already shipped) — penalises an actor who delays a `Signal::Play`. This document covers the *complement*: capturing the cost to **observers** when a convention signal does not unfold as advertised.

## 1. Motivation

H-Group conventions are interpretation-stacked: a single clue can match multiple techs ranked by priority (`SAVE < SIMPLE_PLAY_CLUE < PROMPT < FINESSE`). When a higher-priority interpretation is provisional (e.g. a finesse hypothesis that requires the finessed player to blind-play next turn), the lower-priority interpretation acts as the **fallback**: if the trigger rejects, the receiver should re-resolve the clue under the next-best interpretation.

The current engine implements only half of this:

- ✅ `PendingTrigger::BlindPlay` rejects the provisional hypothesis when the finessed player takes a non-Play action (`player_knowledge.rs:175-222`).
- ❌ After rejection the receiver's effective belief falls back to **baseline only** (Good-Touch narrowing). The H-Group fallback interpretation — "if not a finesse, then a direct play / delayed play / save / chop-focus" — is not installed.
- ❌ Nothing in the search penalises states where a receiver's effective belief diverges from the truth, so the cost of a broken convention is invisible to the evaluator beyond the lost tempo of the failed finesse itself.

### Concrete example (`tests/scenarios/search/avoid_stealing_finesse`)

Alice clues rank-2 to Donald as a finesse (focus = Donald's chop r2; Cathy's slot 1 = r1 on finesse position). Cathy fails to blind-play (gives a different clue). Under H-Group:

1. The finesse hypothesis collapses on Donald: slot 4 is no longer R2.
2. The next-best interpretation of the rank-2 clue on chop with chop unplayable is **2-Save**. But Cathy's failure-to-play also rules out *that* reading from being "what the giver meant", because the giver doesn't pause-then-give-a-2-save — they would have signalled save by not requiring Cathy to act.
3. In real play, Donald is now lost and will likely **bomb** if forced to act, or assume the lowest-risk interpretation (chop = trash) and discard.

The engine currently models step 1 (drop hypothesis) but neither 2 nor 3. The simulated Donald ends up with a wide "any-red" mask and benignly avoids playing the card. The search therefore *under-estimates* the damage a broken finesse causes.

## 2. Goal

Make the search treat communication breakdowns as costly: a clue that has more than one viable interpretation, whose top interpretation gets rejected, should leave a measurable scar — both on the observer's knowledge and on the evaluator's score. The fix should encompass:

1. **Fallback interpretation installation** — when the top-priority hypothesis is rejected, install the *next-priority* tech's hypothesis (the one that would have applied if the finesse hadn't existed).
2. **Misinformation accounting** — when a receiver's effective belief excludes the true identity (or commits to a wrong identity), surface that as a leaf-evaluator term so the search avoids reaching such states.
3. **Optional: bomb modeling** — when a player has a known-playable belief that is in fact unplayable, the simulated action should reflect the strike outcome.

## 3. Current Architecture (relevant pieces)

| Concern | File | Key API |
|---|---|---|
| Hypothesis cohorts | `src/engine/knowledge/knowledge_update.rs` | `Hypothesis`, `TrackedHypothesis`, `PendingTrigger`, `KnowledgeUpdate` |
| Per-player narrowings + signals | `src/engine/knowledge/player_knowledge.rs` | `apply_cohort`, `resolve_pending`, `effective_inferred_mask`, `has_play_signal` |
| Per-tech hypotheses | `src/engine/convention/hgroup/tech/*.rs` | `ClueTech::clue_knowledge_updates`, `matches_clue` |
| Dispatcher (priority filter) | `src/engine/knowledge_aware_game_state.rs::collect_hypotheses` | keeps only the highest-priority tier that matches |
| Per-action knowledge application | `src/engine/knowledge_aware_game_state.rs::apply_clue` | runs `collect_hypotheses` for each observer, stores under a single `cohort_id` |
| Evaluator | `src/engine/evaluator.rs` | leaf terms + per-action `clue_precision_bonus`, `signal_ignored_penalty` |

Note: `collect_hypotheses` currently **drops** lower-priority hypotheses entirely. The fallback interpretation we want for misinformation modeling is exactly one of those dropped lower-priority hypotheses, so we need a way to retain them in a *latent* / *dormant* form.

## 4. Proposed Mechanism

### 4.1 Two-tier cohort structure

Extend `Hypothesis` cohorts with **fallback hypotheses** alongside the **primary** ones:

```rust
pub struct Cohort {
    pub primary: Vec<Hypothesis>,      // current behaviour: highest matching priority tier
    pub fallback: Vec<Hypothesis>,     // next matching tier(s), held dormant
}
```

The dispatcher (`collect_hypotheses`) should keep collecting tiers past the first match, stopping at either:
- a fixed depth (e.g. up to two tiers), or
- when an unconditional hypothesis is reached (a non-finesse interpretation will never need a further fallback).

Only `primary` is applied to the receiver's effective state. `fallback` is held for promotion.

### 4.2 Trigger semantics

Extend `resolve_pending`:

- **Confirm**: as today — fallback discarded, primary baked into baseline.
- **Reject**: instead of just dropping the primary hypothesis, **promote** `fallback` to a new primary cohort. If `fallback` is empty, the cohort dies as today.

Concretely, in `player_knowledge.rs::resolve_pending` where the `rejected_ids` branch is taken, look up the cohort's stored fallback and re-apply it. This will produce new narrowings/signals — possibly with their own provisional triggers — for the receiver and observers.

Implementation note: the existing code path stores hypotheses in a flat `Vec<TrackedHypothesis>`. We can either:
- (a) Add a `fallback: Vec<TrackedHypothesis>` parallel field on `PlayerKnowledge`, keyed by cohort_id, or
- (b) Tag each `TrackedHypothesis` with a `tier: u8` and treat tier > 0 entries as dormant until promoted.

(b) is less invasive; (a) is more explicit. Recommendation: (b).

### 4.3 Misinformation leaf term

Add a new `DefaultEvaluator` field:

```rust
pub misinformation_weight: f64,
```

The term scores, for each own-hand card of each player:
- `+0` if the effective inferred mask contains the truth.
- `+misinformation_weight` if the mask **excludes** the truth (committed to a wrong identity).
- `+misinformation_weight * fraction` for partial misinformation where the truth is in the mask but the player's most-likely interpretation under convention is wrong.

The truth source is whichever other player can see the card directly (any `team_knowledge.player(p).visible_cards` bit). In the search this is always available because we generate the simulated cards. For positions where the truth is genuinely unknown to all players (a freshly-drawn deck card with no clues), no misinformation can be assessed and the term contributes zero.

Add a matching breakdown field on `ScoreBreakdown`.

Suggested default weight: between `efficiency_weight` and `lost_score_ceiling_weight` — strong enough to outweigh tactical micro-edges, weak enough that a single transient desync doesn't dominate the leaf. Start at **3.0** and tune.

### 4.4 Bomb modeling (optional, follow-on)

If a player's effective inferred mask collapses to a single identity that is in fact unplayable, and the player executes `Play` on that card during simulation, the engine should advance the table state's strikes counter and discard the (now-revealed) card. This is the most realistic model of a broken finesse but requires plumbing the truth into `apply_play` more aggressively. Stage 1 (fallback + misinformation term) already captures most of the cost without this.

## 5. Files to Touch

| File | Change |
|---|---|
| `src/engine/knowledge/knowledge_update.rs` | Add `tier: u8` to `TrackedHypothesis` (default 0 = primary). Keep `Hypothesis` unchanged. |
| `src/engine/knowledge_aware_game_state.rs::collect_hypotheses` | Stop using `best_priority` as a hard break; instead emit hypotheses tagged with their tier rank within the cohort. Cap at 2 tiers. |
| `src/engine/knowledge_aware_game_state.rs::apply_clue` | Pass tier info through `apply_cohort`. |
| `src/engine/knowledge/player_knowledge.rs` | `effective_inferred_mask` / `has_play_signal` / `apply_baseline_update` must ignore `tier > 0` entries until promotion. `resolve_pending` promotes tier-1 to tier-0 on rejection (instead of just dropping). |
| `src/engine/evaluator.rs` | Add `misinformation_weight` field, `misinformation_score()` helper, breakdown wiring. |
| `src/engine/evaluator.rs` (`ScoreBreakdown`, lines 36–65) | New field + display formatting. |
| `tests/scenarios/search/avoid_stealing_finesse` | Test should now pass — the TwoSave-first line is no longer the only path with 5 plays, and the play-g2-then-Bob-finesse line is shielded from "Cathy could stall" because both the urgency penalty (existing) and the misinformation cost (new) close that escape hatch. |

## 6. Test Plan

1. **Unit: tier-1 hypothesis is dormant.** After applying a finesse to Donald, his `effective_inferred_mask` excludes the tier-1 (DirectPlayClue) interpretation. Adding the tier-1 hypothesis to the cohort does not change `effective_inferred_mask` unless promoted.
2. **Unit: rejection promotes fallback.** Provide a clue that has SimpleFinesse (primary) + a hand-fabricated tier-1 mock. Reject the trigger; check that the receiver's effective mask collapses to the tier-1 mask, not to baseline.
3. **Unit: misinformation evaluator term.** Build a state where player 0's hand has a card whose effective inferred mask is `{R1}` but truth is R2 (i.e. visible_cards in another player shows R2). Confirm `misinformation_score()` returns a non-zero value.
4. **Regression: `does_not_steal_a_finesse_that_bob_could_give`.** With promotion installed, Donald's slot 4 collapses to {R1} after Cathy fails to blind-play; the simulated Donald then plays his slot 4 as r1 → bombs (or under stage-1 just looks playable but is misinformed). Either way the leaf score for the SimpleFinesse-without-immediate-blind-play branch drops below the alternative.
5. **Regression: `prefers_more_efficient_finesse_over_direct_play_clue`.** Direct play and finesse remain near-tied; misinformation cost is zero in both lines (no convention break), so the test still depends on other evaluator tuning.

## 7. Risks and Open Questions

- **Cohort explosion.** Two tiers per cohort doubles the worst-case hypothesis count. Bound the depth at 2 and document why; if needed, drop tier-1 after a turn or two via a `deadline_turn` like signals already use.
- **Tier-1 selection.** Which technique counts as the "fallback"? Today the dispatcher takes only the highest-priority *matching* tier. The fallback should be the next-highest *matching* tier, not just the next-numbered priority. The dispatcher already iterates in priority order, so we extend it from "break on first match" to "collect first two matching tiers".
- **Truth in search.** The search clones `KnowledgeAwareGameState`, which carries the omniscient `Deck` — truth is reachable. Be careful not to leak truth into the player POVs used for action generation, only into the *leaf evaluator*.
- **Interaction with existing `signal_ignored_penalty`.** The urgency penalty is an action-level cost on the actor; the misinformation term is a leaf-level cost on the resulting state. They compose additively, which is intended: a finessed player who stalls pays both the urgency penalty *and* leaves observers in a misinformed state.
- **Convention coverage.** This document assumes SimpleFinesse is the canonical example. SimplePrompt's hypotheses are unconditional (no provisional trigger), so they don't need fallback. Future techs (Reverse Finesse, Bluff, Layered Finesse) will need to opt into a fallback explicitly via their `clue_knowledge_updates`.

## 8. Suggested Implementation Order

1. Land the `tier: u8` field plumbing **without behaviour change** (tier-1 collected, dormant, never promoted). Verify no regressions.
2. Add the `resolve_pending` promotion step. Now tier-1 hypotheses become active on rejection. Add the unit tests in §6.1–6.2.
3. Add the misinformation leaf term + breakdown. Add unit test §6.3.
4. Re-run search regression tests; tune `misinformation_weight` until the failing scenarios behave correctly without breaking the passing ones.
5. (Optional) Stage 2 bomb modeling — separate PR.
