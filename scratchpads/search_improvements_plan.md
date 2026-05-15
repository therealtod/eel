# Search Engine Improvement Plan

This document outlines several architectural and performance improvements for the Hanabi bot's search engine, primarily focusing on `tree_action_selection_strategy.rs` and `evaluator.rs`.

## 1. Eliminate Memory Allocations in the Search Hot Path

The current search implementation heap-allocates heavily at every node in the search tree. Eliminating these allocations is the single most impactful way to increase nodes-per-second (NPS).

### 1.1 Pre-allocated Triangular PV Table ✅ DONE
**What was done:** Introduced `PvTable` in `tree_action_selection_strategy.rs`. It holds a `Vec<Vec<LineStep>>` where row `d` is pre-allocated to capacity `d`. `best_score_at_depth` now takes `&mut PvTable` instead of returning `Vec<LineStep>`; on improvement it calls `set_pv(depth, step)` which uses `split_at_mut` to copy the child row without re-allocating. One `PvTable` is allocated per root candidate in `scored_actions` (outside the hot path).

### 1.2 Candidate Generation Allocations ✅ DONE
**What was done:** Changed `candidate_actions_with_provenance` to return `SmallVec<[ProposedAction; 20]>` (inline capacity 20, matching the `CANDIDATE_INLINE_CAP` constant). The internal `proposed` accumulator and the dedup `seen` vec are both `SmallVec<[_; 20]>` as well, so the typical path avoids the heap entirely on every recursive call. The one rayon call-site in `scored_actions` (not on the hot path) calls `.into_vec().into_par_iter()` since `SmallVec` does not implement rayon's `IntoParallelIterator`.

## 2. Make/Unmake vs. Clone-and-Apply

**Current state:** The search uses a clone-and-recurse model (`let mut next = state.clone(); next.apply(...)`). `KnowledgeAwareGameState` contains `TeamKnowledge`, which in turn holds arrays of `PlayerKnowledge`. `PlayerKnowledge` contains heap-allocated `Vec<TrackedHypothesis>` and `SmallVec`s for signals.
**Improvement:** Implement a Make/Unmake (or Do/Undo) architecture.
- When applying an action, return an `UndoToken` containing the minimal information needed to restore the state (e.g., the previous length of the `hypotheses` vector, the old `own_hand` bitmasks, and any modified `inferred_identities`).
- Instead of cloning the entire state, mutate it in place and then use the `UndoToken` to restore it after the recursive call returns.
- *Alternative:* If full unmake is too complex due to convention resolution, implement Copy-on-Write (CoW) or ensure that `TeamKnowledge` is 100% stack-allocated (by using static arrays instead of `Vec<TrackedHypothesis>`).

## 3. Branch and Bound / Upper Bound Pruning ✅ DONE

**What was done:** Added `upper_bound` to the `Evaluator` trait (default: `f64::INFINITY`, so existing implementations are unaffected). `DefaultEvaluator::upper_bound` computes an optimistic ceiling using `max_achievable_score` (max game score), zero penalties (efficiency, ceiling loss, misinformation), max pace clamped at `number_of_players`, max clue tokens at `harmonic(8)`, and max immediate bonuses of `(play_progress_weight + clue_precision_weight * total_cards) * depth`. Inside `best_score_at_depth`, two pruning checks are applied to each candidate: (1) **per-candidate pruning** — before recursing, if `evaluator.upper_bound(next_state, depth-1) + immediate <= best`, the candidate is skipped entirely; (2) **early exit** — after updating `best`, if `best >= node_ceiling` (the upper bound for the current node), the candidate loop breaks immediately. A `candidate_pruned` trace event is emitted for pruned candidates.

## 4. Transposition Table (TT)

**Current state:** The search evaluates identical states reached via different move orders independently. 
**Improvement:** Implement a Transposition Table.
- Compute a Zobrist hash for `TableState` and the effective inference masks in `TeamKnowledge`.
- Store the hash, the remaining depth, the best score, and the best move in a lock-free hash table.
- At the start of `best_score_at_depth`, check the TT. If a valid entry with `entry.depth >= current_depth` is found, return `entry.score` and prune the search.
- Even if the depth is insufficient for a hard cutoff, the TT provides the best move from previous visits, allowing us to evaluate the best move first (which maximizes the effectiveness of Branch and Bound pruning).

## 5. Floating Point Arithmetic in Evaluator ✅ DONE

**What was done:** Added two module-level lookup tables built by `const fn`: `RECIPROCAL_LUT: [f64; 65]` (1/n for n=0..=64, with index 0 = 0.0) and `HARMONIC_LUT: [f64; 9]` (H(n) for n=0..=8). Replaced the per-card `f64` divisions in `critical_cards_in_hand`, `misinformation_score`, and `clue_demand` with `* RECIPROCAL_LUT[popcount]` table lookups. Replaced the `harmonic` loop (`Σ 1/i`) with a direct `HARMONIC_LUT[n]` index. In `team_empathy_score`, hoisted the division `1.0 / max_identities` out of the inner loop as `inv_max` and replaced `/ max_f` with `* inv_max`.
