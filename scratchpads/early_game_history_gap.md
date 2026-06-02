# Early Game detection: the `history` population gap

## Summary

The Early Game machinery is fully implemented but **structurally inert**: nothing in the
codebase ever calls `KnowledgeAwareGameState::record_snapshot`, so the private `history:
Vec<GameStateSnapshot>` is always empty. As a consequence:

- `is_early_game(history, current, static_data)` walks an empty history, finds no chop-discard
  transition, and **always returns `true`**.
- The new Early Game restriction (`GameActionFilter::early_game_chop_discard`, applied in
  `candidate_actions_with_provenance`) is therefore **always active**: the bot suppresses a
  `DiscardChop` whenever a productive, GTP-compliant alternative exists — regardless of how far
  into the game it actually is.

That happens to match correct H-Group behavior *for now* (you shouldn't discard your chop in the
opening when you have a productive clue), but the **gating is not real**. The phase never ends,
because the event that should end it is never recorded.

## Evidence

```
$ grep -rn "record_snapshot" --include=*.rs .
src/engine/knowledge_aware_game_state.rs   # definition + doc comments only — zero call sites
```

`from_parts` and `new` both initialize `history: Vec::new()`, the field is private, and no driver,
replay path, or search routine pushes to it.

## Why the search doesn't expose the gap (and why that's fine)

Inside `simulate_forward`, the rollout is clone-and-apply:

```rust
let mut next = state.clone();
next.apply(&chosen.action, convention_set, &mut resolver);
next.advance_turn();
```

Neither `apply` nor `advance_turn` records a snapshot, so even simulated lines never accumulate
history. This means early-game status is effectively **frozen at whatever the root reports**
throughout a search — which is the correct semantics for "are we *currently* in the Early Game?"
We do *not* want a simulated chop discard deep in a line to flip the phase off mid-rollout. So the
search side needs no change; the gap is entirely about the **real game** never recording its turns.

## What needs to happen to close it

The contract (documented on `apply`) is: call `record_snapshot()` **before** each action so
`history[t]` captures the state immediately before turn `t+1`'s action. The missing piece is the
driver/replay loop that advances the *observed* game:

1. Find the loop that applies real (ground-truth) actions to a `KnowledgeAwareGameState` — the
   place using `GroundTruthResolver` / `resolver.advances_real_game()`, as opposed to the search's
   `TruthPovResolver` rollouts.
2. Call `state.record_snapshot()` immediately before each `state.apply(...)` there (and only
   there — never in the search rollout).
3. Confirm clue actions still get `turn = history_len() - 1` correctly (the `apply` doc comment
   ties the clue `turn` field to the snapshot index, so ordering of `record_snapshot` vs `apply`
   matters).

### Candidate call sites to audit

- The replay runner / `ReplayRunner` (see `MEMORY.md` → replay infra).
- Any top-level "play a real game" / self-play driver.
- `reconstruct` paths under `src/engine/replay/`.

## Risk once it's wired up

Today the restriction fires unconditionally. The moment `history` starts populating, the phase
will correctly **end at the first chop discard**, after which `DiscardChop` becomes permissible
again even with a productive alternative. That is a genuine behavior change for mid/late game
decisions. Expect to (re)capture some regression/replay tests once gating goes live — particularly
any scenario where the "right" move is a chop discard while a productive clue still exists but the
game is past the opening.

## Related code

- `src/engine/convention/hgroup/h_group_core.rs` — `is_early_game`, `transition_is_chop_discard`.
- `src/engine/knowledge_aware_game_state.rs` — `record_snapshot`, `history`, `is_early_game()`.
- `src/engine/convention/hgroup/game_action_filter.rs` — `early_game_chop_discard`.
- `src/engine/tree_action_selection_strategy.rs` — `candidate_actions_with_provenance` (filter
  application + `is_early_game` threading).
- `docs/domain/hgroup.md` — "Early Game" terminology + "Early Game restriction" principle rows.
