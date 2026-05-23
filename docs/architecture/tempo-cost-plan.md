# Tempo Cost — Architecture Plan

**Status:** Proposed (not implemented).
**Author / Owner:** TBD.
**Related work:** `play_progress_bonus` (already shipped) — rewards Plays that advance the score within the search window. This document covers the *symmetric* per-ply mechanism for non-Play actions, motivated by a horizon-effect failure in `prefers_to_clue_rank_1_rather_than_picking_up_1s_by_color`.

## 1. Motivation

The search runs at fixed depth `D = number_of_players * 2`. Several leaf-evaluator terms model the cost of discards:

- `pace` (`evaluator.rs:1040-1041`) — the position-property pace metric, weighted by `pace_weight`. Pace = `score + deck_remaining + num_players − max_score`. **Only decreases on discards** (plays keep it flat, clues do not affect it).
- `discard_action_penalty` (`evaluator.rs:1223-1252`) — a *per-event* penalty applied as an immediate bonus when the action is a `Discard`. Combines Bottom Deck Risk and the known-playable-skip penalty.
- `critical_exposure_penalty` (`evaluator.rs:1044-1049`) — discourages letting truth-critical cards sit on chop.

Together these create a strong incentive to **postpone discards past the search horizon**, even when the postponement provides no real-game benefit:

- The two `discard_*` terms only fire on a `Discard` action *inside the window*. A `Discard` pushed to ply `D + 1` contributes 0 to the search score.
- The leaf `pace` term is unchanged across any line that contains no discard — clue-clue-clue-... and clue-... look identical to it.
- So the engine can pad a search line with redundant clues (which don't move `pace`, score, or any of the discard penalties) to push the next forced `Discard` outside the horizon, and reap an artificial score advantage over the line that contains the discard.

This is the classic **horizon effect** from chess engines: the engine "buys" an apparent improvement by deferring an inevitable bad event past the leaf.

### 1.1 Concrete failure: `prefers_to_clue_rank_1_rather_than_picking_up_1s_by_color`

Scenario (`tests/scenarios/search/does_not_slow_down_the_game_due_to_foreseeing_too_many_discards_cause_of_search_horizon/table_state.json`):

- Fresh 5-suit game, 3 players, 8 clue tokens, no plays yet.
- Cathy holds `y1 r1 p3 b5 g1` — four 1s and a `b5`. A rank-1 clue to Cathy touches **three new 1s** (and reveals `p1`-known-via-prior-clue if any), enabling three plays across the team in the next few turns. This is the canonical efficient opener.
- The alternative line is colour-clue-then-colour-clue: pick up the 1s one at a time (1 play setup per clue), padding the search window with extra clue actions.

The expected best action is the rank-1 clue. The engine instead prefers the diluted colour-clue line because:

- Every clue along the colour line keeps `pace` constant.
- The colour line happens to push the team's first forced discard past ply `D`, where the discard penalty would have fired.
- The efficient rank-1 line has slightly tighter pacing at the leaf because the chain of plays is followed (within the window) by a discard turn that *does* fire the penalty.

The horizon arbitrage outweighs the substantive efficiency advantage of the rank-1 clue. The test fails.

### 1.2 Why widening the depth is not enough

Doubling `D` would push the same arbitrage out one ply but would not remove it: the engine would simply pad longer chains of clues to reach the new horizon. Search cost roughly doubles per added ply; the fix is structural, not parametric.

## 2. Diagnosis

The core defect is that the evaluator treats discards as **events** rather than as **position properties**. In real Hanabi:

> Every turn that isn't a play consumes from a finite supply of turns. You need `max_score` plays before the deck (plus the final round) runs out. A clue-heavy turn and a discard turn are *equally costly* in turn-budget terms — both are non-play turns.

The current `pace` formula captures the right shape (it counts non-play turns implicitly via `deck_remaining`), but only updates on actions that draw a card. Plays draw too, so they don't show up either. The result is that `pace` is essentially a discard counter.

What we want is a metric that says: *over the search window, how many turns were spent without producing a play?* The leaf score should be lower for lines that wasted more turns, regardless of whether the wasted turn was a clue or a discard.

## 3. Solution: per-ply tempo cost

Add a new immediate per-ply hook on `Evaluator`:

```rust
/// Immediate per-ply tempo cost.
///
/// Returns a non-positive `Score` (penalty) for any action that consumes a turn
/// without making forward progress on the play stacks. A `Play` that advances
/// the score returns 0 here (its tempo value is already credited by
/// `play_progress_bonus`).
///
/// Fires on every action along the search line — clues, discards, and misplays
/// all incur the cost; only successful plays escape it. This breaks the horizon
/// arbitrage where the engine pads search lines with redundant clues to push
/// discards past the leaf: every padding clue now reduces the line's score by
/// `tempo_cost_weight`.
fn tempo_cost(
    &self,
    _action: &GameAction,
    _pre: &KnowledgeAwareGameState,
    _post: &KnowledgeAwareGameState,
    _pre_phantom_plays: u8,
    _post_phantom_plays: u8,
) -> Score {
    0.0
}
```

`DefaultEvaluator::tempo_cost` implementation:

```rust
fn tempo_cost(
    &self,
    action: &GameAction,
    pre: &KnowledgeAwareGameState,
    post: &KnowledgeAwareGameState,
    pre_phantom_plays: u8,
    post_phantom_plays: u8,
) -> Score {
    if self.tempo_cost_weight == 0.0 {
        return 0.0;
    }
    let variant = &pre.static_data().variant;
    if matches!(action, GameAction::Play { .. })
        && post.score(variant, post_phantom_plays) > pre.score(variant, pre_phantom_plays)
    {
        // Successful Play — tempo-positive; the reward is paid by play_progress_bonus.
        0.0
    } else {
        -self.tempo_cost_weight
    }
}
```

And wire it into `TreeActionSelectionStrategy::immediate_action_bonus` (`tree_action_selection_strategy.rs:190-249`) so it folds into the same per-ply accumulator that already carries `play_progress_bonus`, `clue_precision_bonus`, etc.

`upper_bound` (`evaluator.rs:1368-1410`) stays sound: tempo cost is `≤ 0`, so an optimistic bound that ignores it remains an upper bound.

### 3.1 Why this works

Compare two lines through the failing test, both of length `D`:

- **Efficient (rank-1 clue):** `Clue, Play, Play, Play, Clue, …` — 1–2 clues plus 3 plays inside the window. Tempo cost ≈ `−2 × tempo_cost_weight` (the two clues); plays add `+3 × play_progress_weight` to the line via the existing bonus.
- **Diluted (colour clues):** `Clue, Clue, Play, Clue, Clue, Play` — 4 clues, 2 plays inside the window. Tempo cost ≈ `−4 × tempo_cost_weight`; plays add `+2 × play_progress_weight`.

The diluted line is now strictly worse on the tempo axis, by `~2 × tempo_cost_weight + 1 × play_progress_weight`. The leaf-side trickery (push the discard past `D`) no longer compensates, because the engine has to "pay" for each padding clue on the way to the leaf.

Crucially, the cost is **position-independent**: it doesn't matter *when* in the window the non-play action happens, only how many of them there are. This is the property that closes the horizon hole.

### 3.2 Interaction with existing terms

| Term | Type | Triggers on | Tempo cost overlap |
|------|------|-------------|--------------------|
| `pace` | leaf | discard (via deck-1) | partial — captures real-game pace at the leaf, not search-line tempo |
| `play_progress_bonus` | per-ply | successful Play | complementary — pays the positive side; tempo cost pays the negative side |
| `discard_action_penalty` | per-ply | Discard | additive — discards now pay both the BDR/KP penalty *and* one tempo unit |
| `signal_ignored_penalty` | per-ply | non-Play with active signal | additive — orthogonal axis (convention urgency) |
| `team_empathy_delta_bonus` | per-ply | Clue | partial offset — clues that tighten team empathy claw back some of the tempo cost |
| `clue_precision_bonus` | per-ply | Clue | partial offset — efficient clues earn precision credit |

The intended reading: a clue is always a tempo cost; an *effective* clue earns back enough through `team_empathy_delta_bonus` + `clue_precision_bonus` + downstream plays to be net-positive. A redundant clue stays net-negative.

## 4. Calibration

Tuning targets, in priority order:

1. **Must dominate horizon arbitrage.** Per-ply cost must exceed the maximum spurious gain from postponing one discard by one ply. The dominant discard-related leaf terms are `pace * pace_weight` (worth 1 point per discard at default weights) and `bottom_deck_risk_weight * BDR` (up to ~10 points for a near-empty deck near-critical discard). For mid-deck typical positions the postponed discard's avoided cost is `≈ 1.0–3.0`. **Suggested default:** `tempo_cost_weight = 1.5`.
2. **Must not dominate ceiling preservation.** A save clue costs one tempo unit but prevents a potential `lost_score_ceiling_weight = 8.0` ceiling loss. With `tempo_cost_weight = 1.5`, save clues remain strongly favoured.
3. **Must not over-discount delayed plays.** A delayed-play clue costs `tempo_cost_weight` immediately and earns `play_progress_weight = 1.0` when it unfolds. If the connecting play happens within the window, net = `play_progress_weight − tempo_cost_weight + team_empathy_delta` ≈ `−0.5 + team_empathy_delta`. With typical `team_empathy_delta_bonus ≈ 0.3–0.6`, delayed plays stay near break-even — preferred to discard-padding (`−1.5`) but not to direct plays.

Starting value: **`tempo_cost_weight: 1.5`**. Single integer regression: re-tune `play_progress_weight` upward (toward `2.0`) if delayed-play lines lose out.

## 5. Implementation steps

1. **Trait + default impl** (`src/engine/evaluator.rs`)
   - Add `tempo_cost` method to the `Evaluator` trait with default `0.0`.
   - Add `pub tempo_cost_weight: f64` to `DefaultEvaluator`, default `1.5`.
   - Implement `DefaultEvaluator::tempo_cost` per §3.
   - Extend `ScoreBreakdown` with a `pub tempo_cost: f64` field (parallel to `team_empathy` and `critical_exposure_delta` — accumulated per-line, zero at the pure leaf). Update `Display` impl.

2. **Wire into search line** (`src/engine/tree_action_selection_strategy.rs`)
   - In `immediate_action_bonus` (~line 190): add a call to `evaluator.tempo_cost(...)` and include it in the returned sum.
   - In `scored_actions` (~line 478): the bonus already folds into `immediate_bonus`, no extra wiring needed.

3. **Upper bound** (`src/engine/evaluator.rs:1368-1410`)
   - Tempo cost is `≤ 0` per call, so `upper_bound` does **not** need to subtract anything to stay sound. Add a one-line comment explaining the asymmetry.

4. **Breakdown logging** — confirm `eel::search` debug output (`candidate_scored` event at `tree_action_selection_strategy.rs:508`) renders the new term sensibly; the `LineStep::immediate_bonus` total already aggregates per-ply, so no schema change needed there.

5. **Tests** (see §6).

6. **Doc updates**
   - `evaluator.rs` top-of-`DefaultEvaluator` docstring (`evaluator.rs:280-298`): add tempo to the scoring-terms list.
   - Mention the new term in `docs/architecture/design.md` if/when that doc grows an evaluator section. No other docs reference the evaluator's term list today.

## 6. Testing

### 6.1 Unit tests (`src/engine/evaluator.rs` test module)

- `tempo_cost_zero_for_successful_play` — Play that advances score returns 0.
- `tempo_cost_charges_misplay` — Play that strikes returns `−tempo_cost_weight` (still a wasted turn).
- `tempo_cost_charges_clue` — any Clue returns `−tempo_cost_weight`.
- `tempo_cost_charges_discard` — Discard returns `−tempo_cost_weight` (stacks with `discard_action_penalty` and `pace` — verify both fire in an integration test).
- `tempo_cost_respects_weight_zero` — with `tempo_cost_weight = 0.0`, no contribution.

### 6.2 Regression test — the motivating failure

- `tests/search_regression.rs::prefers_to_clue_rank_1_rather_than_picking_up_1s_by_color` must pass after this change.

### 6.3 Non-regression tests

The full `tests/search_regression.rs` and `tests/replay_regression.rs` suites must continue to pass. Particular attention to scenarios where the engine *should* clue rather than play (saves, finesses with a connecting card that's still over the horizon):

- `defers_playing_a_known_playable_to_save_a_critical_card` — save still preferred (large `lost_score_ceiling_weight` dominates one tempo unit).
- `prefers_more_efficient_finesse_over_direct_play_clue` — finesse still preferred (the finesse triggers a play *inside* the window, claiming `play_progress_weight` back).
- `should_not_steal_a_play_clue_from_bob_and_rather_save_his_chop` — save still preferred over alternative non-action.

If any of these regress at the suggested `tempo_cost_weight = 1.5`, lower toward `1.0` or compensate via `team_empathy_weight` / `play_progress_weight`.

### 6.4 Diagnostic

When debugging via `diag-search-regression`, the per-line `immediate_bonus` total will now include tempo contributions. Make the breakdown explicit in `Display` so a line of N non-play actions shows `tempo=-N*w` at a glance.

## 7. Alternatives considered

### 7.1 Quiescence extension

Extend the search at "non-quiet" leaves (e.g. positions where the next player has 0 clue tokens and must discard). Common technique in chess engines.

**Rejected** as the primary fix because:
- The arbitrage simply migrates to the new horizon — at depth `D + k`, the engine pads lines to `D + k`.
- Search cost grows multiplicatively (each extension adds another branching factor's worth of nodes).
- Doesn't generalise to "the engine wastes clue tokens to dilute pace"; only addresses the boundary case where a discard is unavoidable next ply.

Could be revisited as a *complement* once tempo cost is in place, but is not required.

### 7.2 Depth-aware leaf adjustment

Subtract `tempo_cost_weight × (depth_consumed − plays_in_window)` at the leaf. Mathematically equivalent to the per-ply approach when applied uniformly.

**Rejected** because it requires threading the search depth (and a `plays_in_window` counter) into the leaf evaluator, which currently does not know how many plies it represents. Per-ply accumulation reuses the existing `immediate_action_bonus` plumbing with no new state.

### 7.3 Reform `pace` to decrease on every non-play turn

Change the pace formula itself so that clues also "spend pace". This is the most theoretically pure fix — pace becomes a true tempo metric.

**Rejected for now** because `pace` is also consumed by `required_efficiency`, `clue_demand`, and external diagnostics (see `table_state.rs:241-259` and the test cases in `table_state.rs:660-800`). Redefining it would ripple through many call sites and break the existing Hanabi-community meaning of "pace". Per-ply tempo is additive and reversible (set `tempo_cost_weight = 0.0` to disable). If the per-ply approach proves stable we can consider rolling it into a pace redefinition later.

### 7.4 Increase `discard_action_penalty`

Make the per-event discard cost so large that postponing it by one ply isn't worth it.

**Rejected** because:
- It does not address the symmetric problem (the engine could still dilute *across* lines that all eventually discard).
- Inflating the per-event discard cost distorts the relative value of avoiding a *specific* dangerous discard (BDR, known-playable-skip) versus a routine safe one.
- Doesn't fix lines that postpone the discard *forever* within the window by maxing out clue tokens and idling.

## 8. Open questions

1. **Should a Play that triggers a phantom play (`is_phantom_play = true`) count as tempo-positive?** Likely yes — `play_progress_bonus` already rewards it (`evaluator.rs:1216`), and the team's effective score moved. Implementation should use `post.score(variant, post_phantom_plays) > pre.score(variant, pre_phantom_plays)` exactly as `play_progress_bonus` does, for consistency.
2. **Discard-while-known-playable double-charging.** This case already eats `discard_while_known_playable_penalty = 8.0` plus the BDR. Adding `+1.5` tempo on top is small but worth confirming via the replay suite that it doesn't tip any decision.
3. **Should the per-clue tempo cost scale with `clue_token_bank` saturation?** When tokens are abundant, a "wasted" clue is cheaper. Out of scope for v1 — start with a flat cost; revisit if a regression suggests it matters.
