# Search Engine Improvement Plan

This document outlines several architectural and performance improvements for the Hanabi bot's search engine, primarily focusing on `tree_action_selection_strategy.rs` and `evaluator.rs`.

## 1. Eliminate Memory Allocations in the Search Hot Path

The current search implementation heap-allocates heavily at every node in the search tree. Eliminating these allocations is the single most impactful way to increase nodes-per-second (NPS).

### 1.1 Pre-allocated Triangular PV Table ✅ DONE
**What was done:** Introduced `PvTable` in `tree_action_selection_strategy.rs`. It holds a `Vec<Vec<LineStep>>` where row `d` is pre-allocated to capacity `d`. `best_score_at_depth` now takes `&mut PvTable` instead of returning `Vec<LineStep>`; on improvement it calls `set_pv(depth, step)` which uses `split_at_mut` to copy the child row without re-allocating. One `PvTable` is allocated per root candidate in `scored_actions` (outside the hot path).

### 1.2 Candidate Generation Allocations
**Current state:** `candidate_actions_with_provenance` builds a `Vec<ProposedAction>` using `flat_map` and `.collect()`, then deduplicates it using `Vec::retain` and another `Vec<GameAction>`.
**Improvement:** Replace `Vec` with `smallvec::SmallVec` or `arrayvec::ArrayVec`. The number of candidate actions in Hanabi is typically small (< 20). Generating and deduplicating actions entirely on the stack will bypass the memory allocator.

## 2. Make/Unmake vs. Clone-and-Apply

**Current state:** The search uses a clone-and-recurse model (`let mut next = state.clone(); next.apply(...)`). `KnowledgeAwareGameState` contains `TeamKnowledge`, which in turn holds arrays of `PlayerKnowledge`. `PlayerKnowledge` contains heap-allocated `Vec<TrackedHypothesis>` and `SmallVec`s for signals.
**Improvement:** Implement a Make/Unmake (or Do/Undo) architecture.
- When applying an action, return an `UndoToken` containing the minimal information needed to restore the state (e.g., the previous length of the `hypotheses` vector, the old `own_hand` bitmasks, and any modified `inferred_identities`).
- Instead of cloning the entire state, mutate it in place and then use the `UndoToken` to restore it after the recursive call returns.
- *Alternative:* If full unmake is too complex due to convention resolution, implement Copy-on-Write (CoW) or ensure that `TeamKnowledge` is 100% stack-allocated (by using static arrays instead of `Vec<TrackedHypothesis>`).

## 3. Branch and Bound / Upper Bound Pruning

**Current state:** The comments in `best_score_at_depth` state that "No alpha-beta pruning is performed" because it's a cooperative maximizing search and lacks an upper bound. 
**Improvement:** While there are no min-nodes, we can still prune using Branch and Bound.
- We can construct a strict upper bound heuristic: `upper_bound(state) = current_score + (depth * max_immediate_bonus) + bounds_on_future_gains`. 
- `DefaultEvaluator::max_achievable_score` already calculates the theoretical maximum game score achievable from a given discard pile. We can formulate a max possible `ScoreBreakdown` (max pace, max clue tokens, zero penalties).
- If `upper_bound(state) <= best_score_so_far`, we can immediately prune the subtree, knowing it can never surpass our already-found principal variation.
- **Early Exit:** If the search finds a line that achieves the absolute maximum theoretical score of the game, it can immediately short-circuit.

## 4. Transposition Table (TT)

**Current state:** The search evaluates identical states reached via different move orders independently. 
**Improvement:** Implement a Transposition Table.
- Compute a Zobrist hash for `TableState` and the effective inference masks in `TeamKnowledge`.
- Store the hash, the remaining depth, the best score, and the best move in a lock-free hash table.
- At the start of `best_score_at_depth`, check the TT. If a valid entry with `entry.depth >= current_depth` is found, return `entry.score` and prune the search.
- Even if the depth is insufficient for a hard cutoff, the TT provides the best move from previous visits, allowing us to evaluate the best move first (which maximizes the effectiveness of Branch and Bound pruning).

## 5. Floating Point Arithmetic in Evaluator

**Current state:** `DefaultEvaluator` uses `f64` heavily, including floating point divisions in tight loops (e.g., `critical_cards_in_hand` and `misinformation_score`).
**Improvement:** 
- Convert the evaluator to use integer arithmetic (e.g., scaled integers like `i32` where 1 unit = 0.01 score) to avoid floating-point latency.
- Alternatively, pre-calculate fractional penalties/bonuses into look-up tables based on popcounts.
