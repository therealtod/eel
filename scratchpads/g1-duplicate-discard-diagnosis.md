# Scratchpad — `should_wait_for_g1_duplicate_to_discard` regression

Status: **FIXED.** `parked_dead_weight_penalty` raised `6.0 → 12.0` (candidate fix #1,
parked-dead-weight variant). Target test passes; full suite green (no regressions).

## Chosen fix (applied)

Raised `parked_dead_weight_penalty` from `6.0` to `12.0` in
`src/engine/evaluator/default_evaluator.rs` (and added the missing row to the weight
table in `docs/architecture/evaluator.md`).

Why this lever (not `bad_touch_penalty`):
- The differentiator is **duplicate-waste pricing**, and the g1 line carries exactly
  **1 unit of parked dead weight** in the real search leaf (the duplicate deck[9], dead
  once the green chain plays); the red line carries **0** (it is unchanged under
  `parked_dead_weight_penalty ×3`). So the lever bites *only* the offending line.
- It is **independent of the `bad_touch` reliefs** (forced-save / forced-stall), so the
  recent relief-calibration work is untouched — unlike raising `bad_touch_penalty`, which
  is also far too blunt here (×3 swings the g1 line by 50 points).

Calibration (from the `tests/_diag.rs` sweep): g1 `+53.54` vs red `+50.45`, gap `3.09`,
1 dead-weight unit. Flip threshold is between `9` (g1 still wins by 0.09) and `10` (red
wins by 0.91). Chose `12.0` (2× the prior value) for a ~2.9-point margin.

## The scenario

- Replay: `tests/replays/should_wait_for_g1_duplicate_to_discard.json`, 3 players, No Variant.
- Probe turn: 10 (`engine_action_at_turn(json, 10)`).
- Expected: `Clue { player_index: 2, ClueType::Color, value: 0 }` — red to Cathy.
- Actual: engine clues the **g1** (Rank-1 to Bob), which **bad-touches a duplicate g1**.
- Test author's intent: ignore the duplicate g1; clue the *surviving* g1 only after the
  other copy naturally discards. A duplicate g1 is not worth saving.

### Deck identities (suit-major id = suit*5 + rank-1; suits R,Y,G,B,P)

| deck idx | card | role |
|---|---|---|
| 7 | G1 | Bob's real, genuinely-playable g1 |
| 9 | G1 | the **duplicate** (dead weight; truth-unplayable once 7 plays) |
| 17 | G2 | continues green chain |
| 13 | G3 | continues green chain |
| 18 | R2 | — |
| 12 | R3 | — |
| 0 | B2 | — |

## Original hypothesis (WRONG) — "phantom over-credit" / Save Principle bug

Initial guess: a Save-Principle evaluator term (`save_pressure_known_playable_relief`)
was over-valuing the duplicate, OR a phantom-play hallucination credited *both* g1s as
played, inflating the g1-clue line by ~13 points.

**Both falsified.** See below.

## What the measurements actually show

### 1. Weight-isolation sweep (via `tests/_diag.rs` probe)

- `discard_while_known_playable_penalty = 0` → ranking unchanged.
- `save_pressure_known_playable_relief = 0` → ranking unchanged. **Save-Principle terms
  are NOT the driver.** (Corrected the user's/my terminology mid-investigation.)
- `critical_exposure_weight = 0` → unchanged.
- `bad_touch_penalty * 3` → **flips** the ranking (red wins).
- `parked_dead_weight_penalty * 3` → **flips** the ranking (red wins).

So the differentiator is **duplicate-waste pricing**, not save pressure.

### 2. Policy toggle

`SUBJECTIVE_POLICY = Priority` (instead of `Recursive`) → the g1 clue drops out, red
wins by +35.85. Confirms the **recursive subjective policy** is what elevates the g1 clue.

### 3. Firehose (`--features search-diagnostic`)

There IS a real phantom double-count in the **inner subjective sub-searches**: good-touch
re-narrowing launders deck[9] from G1→P1, and both g1s get "played" (inner leaf score 70).
BUT this lives only in Bob's *decision-making* sub-search.

### 4. ROOT-TRUTH replay of the top PV (the decisive probe)

Replaying each winning PV under a **fixed root-truth resolver** (mirrors how
`simulate_forward` resolves at `at_root_sim=true`, anchored on root player Alice with full
sight of teammates):

| Top PV | phantom_plays | strikes | table_score |
|---|---|---|---|
| **g1 clue** (Rank-1 → Bob) | **0** | **0** | **7** |
| red clue (→ Cathy) | 0 | 0 | 6 |

The g1 line reaches **score 7 with zero phantoms, zero strikes**. Its PV is the legitimate
green chain **G1(7) → G2(17) → G3(13)**; the duplicate deck[9] is **never played** — it
just sits as dead weight.

## Corrected conclusion

1. The "~13-pt phantom over-credit" was an artifact of comparing the search
   (root-truth resolution, score 7) against `replay_leaf_breakdown` (per-actor
   **subjective** resolution, score 5). That 13-pt gap is the **resolution-mode
   difference**, not a hallucination.
2. The phantom double-count is real but **only in inner sub-searches** and does **not**
   propagate to the final score. It also doesn't drive the decision: Bob plays his real
   g1 (deck[7]) regardless of any optimism about deck[9].
3. **The requested truth-aware guard in the `KnownPlayableAmbiguous` branch is
   infeasible leak-safely AND won't fix this test:**
   - That branch is by definition the *no-legitimate-sight* case — if the resolver's
     observer/truth could identify the card, `resolve_play` returns `Known` and the
     branch is never entered. The only "truth" available inside it is the acting player's
     own hidden hand. Using it is exactly the leak the user forbade
     ("the observer's trust cannot leak into other players' reasoning").
   - The winning PV has **no phantom** to guard anyway (proof #4).
4. **Real cause = penalty calibration vs. search horizon.** The g1 clue genuinely buys a
   3-play green chain (→7) *inside* the 6-ply horizon, while the cost of the wasted
   duplicate + wasted clue lands *beyond* the horizon. `bad_touch_penalty` (25) and
   `parked_dead_weight_penalty` (6) don't fully price the long-term waste — which is why
   ×3 of either flips it and the save terms do nothing.

## Candidate fixes (not yet chosen)

1. **Tune duplicate-waste penalties** — raise `bad_touch_penalty` and/or
   `parked_dead_weight_penalty`. Sweep shows ×3 of either flips it. Lock with the new
   regression test. Risk: over-penalizing *forced* bad-touch saves (note the recent
   `feat(eval): forced-stall bad-touch relief` work — calibration must respect it).
2. **Price the parked duplicate at the leaf** — recognize a clue-touched truth-dead
   duplicate as costing a future clue/turn to discard. More principled, more design work.
3. **Re-examine the test premise** — is reaching score 7 here actually wrong? The chain
   is genuinely good; the "wait for the dup to discard" expectation might be the
   questionable part. Worth confirming before changing weights.

## Repro / probes

- Diagnostic harness: `tests/_diag.rs` (gitignored). Contains the empathy/tech dump, the
  weight sweep, `replay_leaf_breakdown` leaf breakdown, and the **ROOT-TRUTH replay**
  probe (the decisive one).
- Run: `cargo test --test _diag diag_g1_dup -- --nocapture`
- Firehose: add `--features search-diagnostic`.

## Key code locations

- `src/engine/knowledge_aware_game_state.rs:437-461` — `KnownPlayableAmbiguous` branch
  ("that branch" the guard was requested in). `phantom_claimed_mask` dedup ledger.
- `src/engine/play_resolver.rs:109-172` — `TruthPovResolver::resolve_play`; phantom branch
  only reached when no legitimate sight identifies the card.
- `src/engine/tree_action_selection_strategy.rs` — `SUBJECTIVE_POLICY` const (line ~42),
  `simulate_forward` root-sim vs sub-search resolvers (~548-633), `pick_subjective_recursive`
  (~1069-1210), `replay_leaf_breakdown` (~1283-1320), `scored_actions` (~1328-1456).
- `src/engine/evaluator/default_evaluator.rs` — `bad_touch_penalty`,
  `parked_dead_weight_penalty`, `save_pressure_known_playable_relief`,
  `discard_while_known_playable_penalty`.

## Leak-safety constraint (must preserve in any fix)

Any truth/observer sight used to *score* outcomes must not flow into a player's reasoning
about their own hidden hand or change their decision-making.
