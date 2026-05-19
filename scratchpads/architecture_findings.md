# Eel Architectural Findings & Improvements

Based on a review of the `eel` project's architecture (`src/game/`, `src/engine/`, and the associated documentation), the structural separation between pure mechanics and strategy is highly effective. However, the alpha-beta search hot paths present several opportunities for significant optimization.

## 1. Transition to an "Apply-and-Undo" Search Model

**Current State:**
The `TreeActionSelectionStrategy::best_score_at_depth` function relies heavily on a clone-and-apply paradigm:
```rust
let mut next = state.clone();
next.apply(&proposed.action, convention_set, truth);
```
While `TableState` was specifically designed to be cheap to clone (avoiding allocations for static rules), `KnowledgeAwareGameState` wraps `TeamKnowledge`. `TeamKnowledge` maintains complex hypotheses and multi-dimensional bitwise tracking for every player. Cloning this structure at every branch inside a recursive search tree wastes memory bandwidth and can trigger hidden heap allocations.

**Proposed Improvement:**
Transition to an **Apply/Undo** architecture, similar to `make_move` and `unmake_move` in highly optimized chess engines. 
- Modify the `apply()` method to mutate the state in-place and return an `UndoToken` that captures only the precise state delta (e.g., previous hand masks, previous stack sizes).
- After evaluating the child nodes, use the `UndoToken` to revert the state to its exact configuration before the action was taken.
- This will completely eliminate node allocations inside the search hot path.

## 2. Transposition Tables (Memoization)

**Current State:**
The engine uses branch-and-bound pruning based on the evaluator's `upper_bound`, but it does not cache the results of deeply evaluated states.

**Proposed Improvement:**
Implement a lock-free Transposition Table (TT) or state cache keyed by the `TableState` hash. In Hanabi, many search paths converge on identical states via different move orderings (e.g., Player 1 discards then Player 2 plays vs. Player 1 plays then Player 2 discards). 
- Because `TableState` implements `Hash` and `Eq` and uses bitfields extensively, it is an ideal, lightweight key for a transposition table.
- Caching bounds and best scores will eliminate massive amounts of redundant sub-tree evaluation, effectively allowing the engine to search deeper in the same amount of time.

## 3. Iterative Deepening & Dynamic Search Depth

**Current State:**
In `TreeActionSelectionStrategy::scored_actions`, search depth is fixed to `(number_of_players * 2)`. A rigid search depth makes the bot highly vulnerable to the "horizon effect," where it might choose to inefficiently delay a discard just to push a negative outcome past the edge of its fixed search depth.

**Proposed Improvement:**
- **Iterative Deepening (ID):** Instead of immediately searching to depth `D`, search progressively to depth `1`, `2`, `3`, ..., `D`. This yields the Principal Variation (PV) early, which can be used to perfectly order moves in deeper searches, vastly improving branch-and-bound cutoffs.
- **Dynamic Search Depth:** Implement logic to dynamically reduce depth on highly constrained, "obvious" branches (like a safe discard of known trash), and extend depth on volatile, complex lines (like an ambiguous finesse play) to ensure accurate evaluation without exploding the search space.

## 4. Refining the `phantom_plays` Abstraction

**Current State:**
`KnowledgeAwareGameState` maintains a `phantom_plays` counter to track successful but ambiguous plays (where the exact identity of the card is deferred so the search stays honest). This prevents the search from assuming a specific stack was advanced.

**Proposed Improvement:**
While the mechanism is clever and mathematically sound, it leaks search-specific evaluation heuristics into the persistent game state wrapper. Move the `phantom_plays` tracking out of `KnowledgeAwareGameState` and strictly into the ephemeral search context or `Evaluator` layer. This keeps the state abstraction strictly tied to derived game truth rather than optimistic search scoring mechanics.

---

## Implementation Plan: Iterative Deepening & Phantom Plays Refactor

### Phase 1: Refining `phantom_plays` (Improvement #4)
1. **Remove `phantom_plays` from State:**
   - Remove the `phantom_plays` field from `KnowledgeAwareGameState`.
   - Remove `phantom_plays()` from its public API.
   - Adjust `KnowledgeAwareGameState::score()` and `pace()` to no longer include `phantom_plays`.

2. **Modify the `apply` Return Type:**
   - Change `KnowledgeAwareGameState::apply` to return an `ApplyResult` enum or a struct (e.g., `ActionOutcome { is_phantom_play: bool }`).
   - Update `apply_play` to return this status when it detects an ambiguous but known-playable scenario.

3. **Track `phantom_plays` in the Search Engine:**
   - Update `TreeActionSelectionStrategy::best_score_at_depth` to accept an accumulated `phantom_plays: u8` parameter.
   - When recursing into candidates, if the applied action was a phantom play, pass `phantom_plays + 1` down the search tree.
   - Modify the leaf evaluation call: `Self::leaf_breakdown(evaluator, state, truth, phantom_plays)`.
   - Update `Evaluator` methods to accept `phantom_plays: u8` so that they can calculate pace and score correctly using the ephemeral search state.

### Phase 2: Iterative Deepening & Dynamic Search Depth (Improvement #3)

1. **Iterative Deepening Search Loop:**
   - Modify `TreeActionSelectionStrategy::scored_actions` to iteratively search from `d = 1` up to `max_depth = (number_of_players * 2)`.
   - Preserve the `PvTable` between iterations. Use the Principal Variation from depth `d-1` to order the root moves at depth `d`. This will maximize branch-and-bound cutoffs.
   - Implement an early exit if a shallower depth already secures the maximum theoretical score.

2. **Dynamic Search Depth Modifiers:**
   - Inside `best_score_at_depth`, implement logic to adjust the `depth` parameter dynamically.
   - **Extensions:** Identify highly volatile or complex lines (e.g., finesse plays or blind plays with high ambiguity). For these branches, pass `depth` (or `depth - 0.5` effectively) instead of `depth - 1`, allowing the engine to see the resolution of the play. (Requires a hard extension cap to prevent infinite loops).
   - **Reductions (LMR):** For highly constrained, low-variance scenarios (e.g., the only viable move is `DiscardKnownTrash`), search the subtree at a reduced depth (e.g., `depth - 2`) after verifying the primary line.

3. **Validation & Benchmarking:**
   - Run unit tests to verify `phantom_plays` scoring remains completely accurate at the leaf nodes.
   - Utilize existing replay integration tests (`cargo test --test '*'`) to ensure Iterative Deepening ordering matches or improves the bot's win rate and decision accuracy.
