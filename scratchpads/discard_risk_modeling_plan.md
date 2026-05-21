# Discard-risk modeling: plan

Context: the regression `prefers_to_clue_rank_1_rather_than_picking_up_1s_by_color`
exposed that the engine treats Alice's unknown chop as ~6 points of risk under
`bottom_deck_risk_score` (BDR), enough to make her prefer slow color-clues over
the efficient rank-1 clue purely because she fears being forced to discard a
turn or two later. The root cause is that BDR (and the broader "discard risk"
modeling) does not distinguish:

- known-critical vs probabilistic-critical
- worst-case vs expected-value risk
- "no teammate intervened, so the team is implicitly OK with this discard"
  (Permission To Discard) vs "the team didn't have a chance to react yet"

This document plans four interlocking changes. They can ship independently; #2
and #3 are the most impactful.

Related work: `scratchpads/critical_exposure_followups.md` covers concentration
and draw-potential refinements to the *leaf* critical-exposure term. The work
here is largely orthogonal but #1 below cross-references it.

---

## 1. Verify and document the certainty-only nature of `critical_exposure_weight`

### Status
Already correct. `critical_exposure_score` (`evaluator.rs:468-520`) only
contributes for cards where `truth.card_identity(deck_idx).is_some()` and that
identity is in `critical_mask`. Unknown/drawn cards contribute zero. The
docstring already says "reflects *known* critical risk, not hypothetical risk
from drawing into a fresh slot."

### Action
- Audit the codepath end-to-end and **add a unit test** asserting that a
  drawn-into hand slot (no truth identity) never contributes to
  `critical_exposure_score`, even if its empathy includes critical ids. This
  pins the contract so future refactors don't accidentally probabilise it.
- Add a one-line comment near the early-continue at `evaluator.rs:508-510` that
  explicitly references the BDR term as the "probabilistic counterpart" so the
  separation between the two terms is discoverable.

### Out of scope here
Concentration and draw-potential refinements to this term — see
`critical_exposure_followups.md`.

---

## 2. Tighter probability model for BDR

### Current model
For each id in the discarded card's empathy where discarding *this copy* would
leave exactly one surviving copy and that surviving copy is not visibly held:

```
score += (stacks_size - rank_idx) / popcount(empathy)
```

Problems:
1. `(stacks_size - rank_idx)` is the **worst-case** loss (entire suit above
   this rank lost) — it ignores that some of those higher ranks may already be
   discarded or otherwise unreachable.
2. `1 / popcount(empathy)` assumes uniform empathy. Real empathy is
   non-uniform when convention signals or partial clue resolution have
   narrowed the distribution unevenly.
3. The expected loss is multiplied by `P(actually critical) = 1`, but the
   loss only realises if the surviving copy is **at the bottom of the deck**
   (or otherwise out of reach by game end). Current formula treats this as
   certain.

### Proposed model
Per-id contribution becomes:

```
P(card_is_id)           // probability we're discarding this id
  * P(surviving_copy_lost_to_deck)   // probability the other copy never reaches a hand
  * loss_if_lost(id, table_state)    // actual achievable-score loss
```

#### Term A — `P(card_is_id)`
- Current: `1 / popcount(empathy)`.
- Refined: prefer the deck's marginal probability conditional on global
  observations. If we already have a `MarginalDistribution` per slot
  (check `team_knowledge` / probability tables), use it directly. Otherwise
  start with the current uniform-over-empathy as a placeholder but extract
  this into a named helper so we can swap implementations without touching
  BDR.

#### Term B — `P(surviving_copy_lost_to_deck)`
This is the dominant correction. The surviving copy of id is somewhere; either
- in a hand (already excluded by the `visible_ids` check), or
- in the discard pile (already excluded by `already_discarded` accounting), or
- still in the deck.

If it's still in the deck, the probability it never gets drawn before game end
is approximately `1 / (deck_remaining + 1)` for a "bottom-deck" model, or more
precisely `(deck_drawn_after_this / deck_remaining)` for a "running-out-of-turns"
model. For early game with a near-full deck, this multiplier is small
(~`1/30`); only late game does it approach 1.

The plan: compute `deck_remaining = table_state.deck.current_size` and use
`p_lost = max(0, 1 - turns_remaining / deck_remaining)` capped to `[0, 1]`,
where `turns_remaining` estimates remaining draws until end-game. The simplest
first cut: `p_lost = 1 / max(1, deck_remaining)` (Laplace-style bottom-card
probability). Document this is conservative; refine later.

#### Term C — `loss_if_lost(id, table_state)`
Currently `stacks_size - rank_idx`. Refine to:
- iterate from `rank_idx` upward and stop at the first rank where all copies
  are already discarded → the suit cap before this discard would have been
  lower already. So `loss_if_lost = max(0, suit_cap_now - suit_cap_after_loss)`.
- this matches `max_achievable_score`'s logic; factor out a shared
  `suit_cap_after_discarding(suit, table_state, hypothetical_id)` helper.

### Expected impact on the regression
Early game with ~30 cards in deck, `p_lost ≈ 1/30 ≈ 0.033`. The ~6-point BDR
on Alice's chop becomes ~0.2 — small enough that the rank-1 line wins on
intrinsic merit. Late game `p_lost` rises and the term re-engages.

### Tests to add
- BDR on a chop with full empathy and full deck: small (< 0.5).
- BDR on the same card when deck is nearly empty: large.
- BDR collapses to zero when surviving copy is visibly held (regression
  test for the existing `visible_ids` short-circuit).
- BDR scales linearly with `bottom_deck_risk_weight`.

### Implementation notes
- Extract a `BdrModel` struct with named fields for terms A, B, C so each can
  be unit-tested in isolation.
- Don't break the existing `bottom_deck_risk_score` signature — it's called
  inside `discard_action_penalty` and from tests.
- The truth-collapse path (`truth.card_identity` returns `Some`) is currently
  a wholesale empathy → singleton replacement. Keep that — when the actor's
  hand is observable to the truth player, we know exactly what they're
  discarding; only term B and C apply.

---

## 3. Permission To Discard (PTD)

### Concept
A card is implicitly *cleared for discard* if a teammate, on a previous turn
since the actor's last action, **had the means and information to save it
and chose not to**. This is the dual of how saves are usually inferred:
instead of "they clued my chop, so it must be critical," we infer "they could
have clued my chop and didn't, so it must not be critical."

This is theory-of-mind one level deep. It's how human H-Group players reason
about whether to fear their chop.

### Inputs
For a candidate discard by player P of their card on slot S at turn T:
- For each teammate Q whose turn occurred in `(last_P_turn, T)`:
  - Reconstruct Q's POV at the time of Q's action.
  - Did Q see P's slot-S card as a `(critical_save_eligible | two_save_eligible)`
    candidate under the active convention set?
  - Did Q have at least one clue token at that moment?
  - Did Q's chosen action save P's slot S? If yes → no implication. If no →
    Q *passed* on the save, contributing PTD evidence.
- If at least one teammate Q passed on the save, P has PTD on slot S.

### Implementation choices

#### Option A — On-demand reconstruction inside the evaluator
At discard-evaluation time, walk back through `table_state.history` (we may
need to extend `TableState` to carry a short suffix of past actions) and
reconstruct each teammate's POV at the moment of their action. Query the
convention layer's save techs to determine `would_have_saved(slot)`.

Pros:
- No new state on `team_knowledge`.
- Self-contained in the evaluator.

Cons:
- POV reconstruction is expensive in a search hot path.
- Requires the evaluator to invoke convention techs, which is currently a
  one-way dependency (conventions know about the evaluator via priorities,
  but the evaluator doesn't call into convention techs).

#### Option B — Convention-layer signal write
When a teammate Q takes a non-save action while a critical-save was available
on next player's chop, the convention layer writes a `Signal::PermissionToDiscard`
on that slot's deck index. The evaluator's BDR / critical-exposure terms read
this flag and zero out their penalty for the marked slot.

Pros:
- Cheap at search time: just a flag lookup.
- Lives where the H-Group reasoning belongs (convention layer).
- Reuses the `Signal` infrastructure already in `team_knowledge`.

Cons:
- New signal type. Need to be careful about lifetime — the signal must clear
  when:
  - the card is touched, or
  - the card is discarded (slot replaced by draw), or
  - a new critical-save becomes available that wasn't available before (e.g.
    a card became critical via someone else's discard).

#### Recommendation
**Option B**, with a small twist: emit the signal as part of the same place
where save-eligible-chops are detected (`CriticalSave` / `TwoSave` / `FiveSave`
techs already need this analysis on the giver's side). When such a tech
*offers* a candidate save and the search decides to pick a different action,
the strategy layer writes the PTD signal on the not-saved chop card before
the next player evaluates. This keeps the convention logic in conventions,
keeps the evaluator dumb, and clears naturally on draw/play/touch.

### Tests to add
- A teammate sees a critical on next player's chop, has clue tokens, picks a
  different action → next player's chop has PTD signal.
- Discarding a chop with PTD signal yields 0 BDR penalty (or
  `bottom_deck_risk_weight * scale_when_PTD`, see Open Questions).
- The PTD signal clears when the slot is touched.
- The PTD signal clears when a critical card identity newly enters the
  team's view between the prior turn and now.
- End-to-end: `prefers_to_clue_rank_1_rather_than_picking_up_1s_by_color`
  passes (PTD reasoning means Alice doesn't fear her chop in the rank-1 line
  because Cathy will pass on saving it next turn — but careful, this is
  *prospective*: the next-turn reasoning is in the leaf, not the immediate
  action). May need #4 as the complementary fix for the immediate-bonus side.

### Open questions
- **Hard vs soft PTD waiver**: should PTD zero out BDR entirely, or just
  reduce it by a factor (e.g. 0.25)? Hard waiver is more in keeping with how
  H-Group treats it. Soft waiver is safer against incorrect PTD inference.
- **Multi-teammate PTD**: in 3p+, multiple teammates may have passed since
  the actor's last turn. Each passed save is independent evidence — does PTD
  strengthen with more passes? Probably not; one pass is enough to clear.
- **PTD across draw boundaries**: if the teammate's pass-save was on slot
  index S but the card at S has since been replaced by a draw, the PTD does
  not apply to the new card. The signal-clearing rules above handle this.

---

## 4. Soft fallback — action-availability scaling

Even with PTD inference, there will be cases where the previous teammate's
turn was the actor's own (no PTD opportunity), or the convention layer hasn't
been extended to emit PTD signals yet. As a complementary safety net, scale
the chop-discard penalty by how much *else* the actor has to do.

### Concept
If Alice's convention layer produces only `DiscardChop` (no positive-value
clue, no known-playable, no signal-honor action), the search has already
exhausted reasonable alternatives. In that case, charging Alice the full BDR
penalty is double-counting: the search would penalise the leaf anyway, and
the immediate penalty is just punishing the inevitable.

### Proposed scaling
```
scale = clamp_01(useful_alternatives_count / typical_alternatives)
penalty_actual = bdr_raw * (alpha + (1 - alpha) * scale)
```

Where:
- `useful_alternatives_count` = number of non-discard candidates the
  convention layer produces with positive expected value (priority above
  `DiscardChop`).
- `typical_alternatives = 2` (rough — to be tuned). 0 alternatives → `scale=0`.
- `alpha ∈ [0, 1]` is the **floor** — even with no alternatives, charge `alpha`
  fraction of BDR. Suggested `alpha = 0.3` so the term doesn't fully vanish
  (we still want some pressure to clue when borderline).

### Implementation notes
- The action set is generated by the convention layer just before the
  evaluator scores actions; pass `alternatives_count` into
  `immediate_action_bonus` so the discard branch can apply the scale.
- Need to filter alternatives by "useful": some convention techs always
  propose at least one candidate (e.g. `BlindPlay` with low priority). The
  filter should be "priority > DiscardChop's priority" — which is exactly
  what the action-selection strategy already orders by.

### Tests to add
- With 0 useful alternatives, discard penalty = `alpha * bdr_raw`.
- With many alternatives, discard penalty ≈ `bdr_raw`.
- Sanity: in the `defers_playing_a_known_playable_to_save_a_critical_card`
  test, Alice has both a known-playable and a critical-save available — so
  `scale` is high, and the critical-save logic dominates as expected.

### Interaction with #2 and #3
- #2 (tighter BDR) reduces the raw value; #4 scales it. Apply #4 multiplicatively
  on top of #2.
- #3 (PTD) zeroes BDR entirely when applicable; #4 is for cases where PTD
  doesn't apply.

---

## Ordering and rollout

1. **#1 audit + tests** — half a day. No behaviour change; pins the contract.
2. **#2 BDR refinement** — the highest-leverage single change. Implement
   term B first (deck-remaining probability) since that's the dominant
   correction. Terms A and C are refinements that can wait.
3. **#4 action-availability scaling** — small, well-contained. Ship after #2
   so the scaled value is on top of the corrected raw value.
4. **#3 PTD signal** — largest change. Requires convention-layer plumbing.
   Tackle last but it's the most conceptually satisfying piece — it makes
   the engine actually reason like an H-Group player about chop safety.

### Gating tests
- `prefers_to_clue_rank_1_rather_than_picking_up_1s_by_color` must pass
  after #2 alone (per the expected-impact estimate above). If not, escalate
  to #4 + #2 combined.
- `defers_playing_a_known_playable_to_save_a_critical_card` must keep
  passing through every change. Add an assertion that this test's score
  margin doesn't narrow by more than X% after each change, to catch
  drift even if the choice stays correct.
- Full regression suite (`cargo test --test '*'`) at every step.

---

## Risks

- **Tuning instability**: changing BDR weight while #4 also exists creates
  two interacting knobs. Document a default profile and avoid retuning
  `bottom_deck_risk_weight` after #2/#4 ship until the regression suite is
  stable.
- **PTD signal lifetime bugs**: signals that fail to clear on draw/touch
  could create false-positive discards. Cover the clearing rules with
  property-style tests.
- **Convention-evaluator coupling**: option B for PTD adds a convention →
  team_knowledge → evaluator dataflow. This is the same pattern as existing
  `Signal::Play`, so the precedent is fine, but the new signal needs to be
  documented in `docs/architecture/overview.md` (per `AGENTS.md` doc-update
  rule).
