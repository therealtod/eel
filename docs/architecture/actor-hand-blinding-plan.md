# Actor-Hand Blinding in Subtree Evaluation — Architecture Plan

**Status:** Proposed (not implemented).
**Owner:** TBD.
**Related work:** Existing `effective_visible` intersection at each ply (`tree_action_selection_strategy.rs:317-327`) — handles the *"deeper player can see more than the root"* direction of the cheating-bot leak. This document covers the *reverse* direction.

## 1. Problem

The fixed-depth search at `tree_action_selection_strategy.rs::best_score_at_depth` lets an actor's own (hidden-from-themselves) hand influence the actor's optimal choice — even though the actor cannot observe their hand.

The mechanism is downstream-leak, not direct-leak:

1. Actor `A` at ply `k` cannot see their own hand. The active POV at ply `k` has `effective_visible = root.visible ∩ A.visible`, which excludes `A`'s own cards. `A`'s immediate-bonus computation honours this. ✓
2. The search recurses into ply `k+1`. The recursive `best_score_at_depth` is called with the **original `truth` pointer** (`tree_action_selection_strategy.rs:386`), so subsequent plies still compute `effective_visible = truth.visible_cards() ∩ active.visible_cards`.
3. At ply `k+1`, the next player `B` legitimately sees `A`'s hand in real life, and their `effective_visible` includes `A`'s cards.
4. `B`'s evaluation of their own action uses `A`'s actual card identities — sometimes via `is_potential_bad_touch`, sometimes via `play_progress_bonus`, sometimes via leaf scoring.
5. That ply-`k+1` score bubbles back up to `A`'s ranking of choices.

Net effect: `A`'s subtree score is a function of `A`'s actual hidden hand. `A` picks the action that is best *given the truth of their hand*, not the action that is best in expectation over what they could be holding.

### 1.1 Concrete reproduction

`tests/search_regression.rs::does_not_defer_play_to_save_cathy_when_bob_could_save` exposes this leak. Repro by running:

```
cargo test does_not_defer_play_to_save_cathy_when_bob_could_save 2>&1
```

Scenario (`tests/scenarios/search/avoid_deferring_play_to_save/table_state.json`):

- 3 players. Alice has a clued g2 (known playable). Cathy's chop is `y3` — a critical save (one `y3` already discarded). Bob's hand contains `g3` (currently playable after Alice's g2 lands).
- Test expects Alice to play g2; Bob will later save Cathy's `y3`.
- Engine instead chooses to give the rank-3 critical save to Cathy herself, then play g2 next turn. The 0.42-point margin between the chosen and expected root actions is downstream of the leak below.

Trace decomposition (recorded with `RUST_LOG=eel::search=trace` and the parallel-iterator at line 474 temporarily replaced by serial `into_iter`):

| Bob's action @ ply 5 | Bob's immediate | Bob's subtree | Bob's score |
|---|---|---|---|
| CriticalSave color-1 [10] (single-card y3 save) | −1.072 | 73.247 | **72.175** |
| CriticalSave rank-3 [12, 10] (two-card save: y3 + r3 incidentally) | −0.892 | **68.491** | 67.599 |

Rank-3 has the *better* immediate bonus (more information per token), but the subtree is 4.76 points worse. Decomposing one further ply, Cathy's optimal turn-2 follow-up — clue color-2 to Bob's g3 — has dramatically different `immediate` across the two contexts:

| Cathy's clue color-2 [6] context | `clue_bonus` | `signal_penalty` | `team_empathy_bonus` | `tempo` | total |
|---|---|---|---|---|---|
| After Bob's color-1 [10] | +0.400 | 0.0 | +0.516 | −2.500 | **−1.584** |
| After Bob's rank-3 [12, 10] | **−4.600** | 0.0 | +0.516 | −2.500 | **−6.584** |

The −5.0 swing is the `potential_bad_touch_penalty` inside `clue_precision_bonus` (`evaluator/scoring.rs:178-194`). It fires because:

- After Bob's rank-3, Cathy's index 12 inference is `{r3, y3, g3, b3, p3}` — wide because no tech narrows the non-chop touched card of a critical save.
- Her `giver_held` (union of clued-card inferences) therefore contains `g3`.
- The clue's `touched_on_receiver = {g3}` because Bob's index 6 *is* `g3` (Cathy's POV legitimately sees Bob's hand).
- Intersection non-empty → `is_potential_bad_touch` returns true → 5.0 penalty.

The penalty firing is *internally consistent* from Cathy's POV — she does see Bob's g3, and her index 12 *could* be g3. The architectural issue is that this penalty is the deciding factor in **Bob's** rank-3 vs color-1 choice, even though Bob has no way of knowing he has g3.

If Bob's index 6 were any other identity (say `r4`), Cathy's color-2 clue wouldn't touch g3 and the penalty wouldn't fire. Bob's ranking would reverse. The choice tracks the truth of his hand.

### 1.2 Why the existing visibility intersection doesn't catch this

`effective_visible = root_visible & active.visible_cards` is asymmetric:

- It correctly prevents the deepest active player from seeing cards the root can't (the original cheating-bot scenario).
- It does **not** restrict what the active player can see *about other players whose subtree is being explored*. Cathy at her own ply correctly sees Bob — she should — but the search uses *that visibility* to compute Bob's optimal choice at the prior ply.

A subtree-local truth restriction is needed, not just a per-ply POV restriction.

## 2. Proposed fix: blinding the actor's hand from their own subtree

When the search recurses into the subtree rooted at actor `A`'s choice, all downstream player POVs should be computed as if `A`'s hand identities were *not known* — even to players who would legitimately see them in a single-position game.

Concretely, when calling `best_score_at_depth` recursively after an action by actor `A`:

1. Build a `subtree_truth` POV whose `visible_cards` excludes `A`'s own deck indices, *and* whose `card_identity(idx)` returns `None` for any `idx` in `A`'s hand.
2. Pass `subtree_truth` to the recursive call instead of the unchanged `truth`.
3. Downstream `effective_visible = subtree_truth.visible & player.visible` will then also exclude `A`'s hand, propagating the blinding all the way to the leaf.

### 2.1 Where this changes things in practice

`is_potential_bad_touch` at deeper plies will see `touched_on_receiver` shrink whenever a touched card is in the blinded actor's hand: the function uses `truth.card_identity(idx)`, which would now return `None`, and the card contributes 0 to `touched_on_receiver`. The penalty stops firing on cards whose specific identity the actor couldn't possibly know.

`count_bad_touches` (same module) does the same identity lookup and should be similarly affected — without blinding, downstream players' "bad touch" judgements include known-from-truth identities that the actor at the top of the subtree couldn't have anticipated.

Leaf evaluator terms that consult truth — `known_playable_in_hands` (`evaluator/default_evaluator.rs:401`), `critical_exposure_penalty` (per-card identity), `misinformation_score` — fall into the same category and should be reviewed for sensitivity to the blinded actor's hand.

### 2.2 Where this is wrong on its face

The blinded POV is a *fiction*: in actual play, downstream players really do see the actor's hand. Blinding them models "what the actor expects the team to do in expectation over the actor's own uncertainty," not "what the team actually does."

This is acceptable if the search's purpose is to choose the actor's best action under their own knowledge — which is the spirit of cooperative Hanabi convention play. It is incorrect if the search is supposed to predict the team's actual outcome.

For root action selection, the first interpretation is the right one. For evaluator calibration against ground-truth game outcomes, the second is needed. The search currently conflates both roles. This plan picks the first.

### 2.3 Performance considerations

`LightweightPlayerPOV` is cheap to construct; the blinding amounts to one additional bitmask AND on `visible_cards` before passing to the recursive call. No new allocations needed if the blinded POV reuses the same backing storage (`active_pk`, `team_knowledge`, etc.).

Branch-and-bound pruning may behave differently: blinded subtrees often look worse than truth-aware ones (some penalty avoidance no longer happens), so `node_ceiling`s may need recalibration. Expect transient regressions in the existing search-regression test suite that will need scenario-by-scenario re-validation.

## 3. Touch points

- `src/engine/tree_action_selection_strategy.rs`
  - `best_score_at_depth` (entry around line 289): construct `subtree_truth` blinding the just-acted player's hand. Pass it as the `truth` parameter in the recursive call at line 386.
  - Carefully preserve the existing per-ply `effective_visible` intersection at line 319 — it still serves its original purpose (preventing the deeper player from seeing root-hidden cards). Blinding is *additive*.
- `src/engine/knowledge/lightweight_player_pov.rs`
  - Likely need a constructor variant `with_visible_cards_and_blinded(blinded_indices)` or similar, that excludes the blinded indices both from `visible_cards()` *and* from `card_identity()` lookups. Document the invariant: blinded indices behave as if the holder were unknown.
- `src/engine/evaluator/scoring.rs::clue_precision_bonus` and `h_group_core.rs::is_potential_bad_touch` / `count_bad_touches`
  - No code changes expected, but verify behaviour with the blinded POV: when `truth.card_identity` returns `None` for a touched card, the function should naturally treat that card as "unknown identity, can't be flagged as bad touch." Add focused unit tests for this case.
- `tests/search_regression.rs`
  - The current failing test `does_not_defer_play_to_save_cathy_when_bob_could_save` should pass once blinding is implemented (Bob's rank-3 vs color-1 ranking no longer depends on his own g3).
  - The previously fixed tests should be re-verified — blinding may surface new symptoms or fix unrelated latent issues.

## 4. Open questions

- **Scope of blinding:** blind only the *direct actor* of the top-of-subtree move, or also blind every actor *upstream* of the current ply? The conservative choice is to blind only the just-acted player at each recursion level (i.e., at ply `k+1`, blind the player whose action led here from ply `k`). A more aggressive option is to maintain a running set of blinded players, one per recursion level. The minimal version is sufficient for the bug described here.
- **Interaction with `apply` and `TruthPovResolver`:** the `next.apply` call at line 346 also takes `truth`. If the application of an action needs to resolve hidden-identity outcomes (e.g., what card was drawn after a discard), it must use the *unblinded* truth, since the deck draw is determined by the actual game state. Confirm by reading `KnowledgeAwareGameState::apply`.
- **Effect on parallel root iteration:** the `into_par_iter` at line 474 must continue to work; blinding is computed per-recursion-frame and doesn't share state across root candidates.
- **Marginalization vs. blinding:** the *theoretically correct* fix is to marginalize the subtree score over the actor's possible hand identities, weighted by their posterior. Blinding is an approximation: it replaces "average over my possible hands" with "compute as if no one knows my hand." For most decisions where the actor's hand identity is symmetric across plausible choices, the approximations match. Cases where they diverge (e.g., the actor's hand has cards whose identity strongly affects downstream play viability) need a follow-up. Document any such cases discovered during implementation.

## 5. Acceptance criteria

1. `does_not_defer_play_to_save_cathy_when_bob_could_save` passes.
2. Full `cargo test --test search_regression` passes (or any new failures are diagnosed and triaged as evaluator-calibration follow-ups, not blinding bugs).
3. Replay-regression suite (`cargo test --test replay_regression`) still passes.
4. New unit tests in `is_potential_bad_touch` / `count_bad_touches` cover the case where `truth.card_identity(touched_idx)` returns `None`, asserting that the function does not flag the card as bad-touched.
5. A new search-regression scenario (`tests/scenarios/search/actor_hand_blinding/`) demonstrates a case where the *unblinded* search would pick a different root action than the blinded one, asserting that blinding produces the H-Group-correct choice.

## 6. Out of scope

- Full marginalization over actor hand uncertainty (Bayesian average of subtree scores). Track as a follow-up if blinding turns out to be an inadequate approximation.
- Changes to the H-Group convention-tech `knowledge_updates` (e.g., having `CriticalSave` narrow the non-chop touched cards). The localized alternative (option A in the diagnosis conversation) is the proper place for that — it addresses the *calibration* of `is_potential_bad_touch`, which is independent from this architectural fix.
- Changes to the leaf evaluator's truth-using terms (`known_playable_in_hands`, `misinformation_score`, etc.) beyond verifying that they behave correctly with `card_identity` returning `None`. If a term's semantics specifically *require* truth (e.g., misinformation needs to know the actor was clued a wrong identity), document the contradiction with blinding and defer.
