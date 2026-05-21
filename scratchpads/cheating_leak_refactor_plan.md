# Fix the search "cheating bot" leak by restoring the original split between `visible_cards` and `inferred_identities`

## Background

In `should_clue_rank_3_instead_of_red` (replay regression, turn 35), Bob picks
a red play clue to Alice even though that clue is ambiguous between PlayClue
and CriticalSave from Alice's POV (r4 is critical). A rank-3 clue would be
unambiguous and Alice would play immediately.

Diagnostic (`scripts/diag.sh should_clue_rank_3_instead_of_red.json 35`) showed
the search PV for the red clue includes Alice cluing **rank-3 to Bob's actual
rank-3 cards** at a later ply, and Bob then playing slot 1. The total score
beats the rank-3 PV (145.53 vs 134.52) precisely because the search "finds a
recovery" — but the recovery depends on Alice knowing exactly which of Bob's
cards are rank-3, which Bob himself does not know.

This is the classic "cheating bot" leak: the search lets teammates use
omniscient knowledge during rollouts.

## Root cause

`PlayerKnowledge` has two fields that were originally intended to model
distinct things, but their semantics have drifted and merged during
development:

- `visible_cards` — originally meant to record "cards this player can directly
  see." Now also gets set by `narrow_inferred` / `exclude_inferred` whenever
  clue/convention narrowing collapses empathy to a singleton.
- `inferred_identities` — originally meant for "what this player has deduced
  from clue history and convention principles (good-touch, etc.)." Now also
  gets a singleton baked in by `update_with_revealed_card` at draw time
  (direct sight written into the inferred mask).

On top of that, `PlayerKnowledge::combined_possible_identities` consults a
*third* truth source: `table_state.deck.get_global_empathy`, the omniscient
deck.

When `TreeActionSelectionStrategy::best_score_at_depth` switches to a
teammate's turn during the rollout, it does:

```rust
let pov = state.player_pov(active);
```

…which constructs the teammate's POV from `team_knowledge.player(active)`.
That entry was populated with the truth about every other player's hand at
draw time. So a teammate in the rollout reads Bob's hand identities directly
and constructs targeted clues Bob himself could not have predicted.

## Target architecture

Restore the original intent of the two fields:

- `PlayerKnowledge::visible_cards` — **only** cards this player can directly
  see. Mutated by `update_with_revealed_card` (and similar reveal paths). Not
  touched by clue narrowing.
- `PlayerKnowledge::inferred_identities` — **only** clue and convention-
  derived narrowings (good-touch principle, etc.). Not touched by direct
  sight.

"This player knows this card's identity" becomes a derived predicate:
`visible_cards bit set ∨ inferred_identities[i].is_exactly_known()`.

The search override lives at the POV layer. `LightweightPlayerPOV` carries
its own effective `visible_cards` field — for normal POV construction it
defaults to `knowledge.visible_cards`; for `teammate_pov` during search it
intersects with the root POV's effective sight. By construction, Alice's
search-time view of Bob's hand collapses to empty (Bob can't see his own
hand, so the intersection is empty there), and Alice's reasoning falls back
to public clue knowledge only.

## Plan

### Phase 1 — POV-level `visible_cards` override (fixes the leak)

Phase 1 makes no change to `PlayerKnowledge`. It only adds an override layer
at the POV.

- Add `visible_cards: DeckCardsBitField` to `LightweightPlayerPOV`.
- `LightweightPlayerPOV::new` defaults the field to `knowledge.visible_cards`
  (existing call sites keep working unchanged).
- `combined_possible_identities` (or its callers) consults the POV's
  `visible_cards` instead of going through `PlayerKnowledge::visible_cards`.
  The deck-truth fallback fires only for cards inside the POV's effective
  sight.
- Add `LightweightPlayerPOV::teammate_pov(target)` that returns a POV with
  `visible_cards = self.visible_cards & team_knowledge.player(target).visible_cards`.
  Chained calls compose correctly.
- `best_score_at_depth` switches to:
  ```rust
  let root_player = truth.player_index();
  let root_now = state.player_pov(root_player);
  let pov = if active == root_player {
      root_now
  } else {
      root_now.teammate_pov(active)
  };
  ```

After Phase 1, `should_clue_rank_3_instead_of_red` passes. The PK still has
the field-mixing drift, but it no longer matters during search because the
POV controls effective sight.

This is the smallest self-contained change. Stop here if budget is tight.

### Phase 2 — clean up `PlayerKnowledge` back to the original split

- `narrow_inferred` and `exclude_inferred` stop bumping `visible_cards` when
  narrowing to a singleton. They mutate `inferred_identities` only.
- `update_with_revealed_card` continues to set `visible_cards`, but stops
  writing the singleton into `inferred_identities`. (Direct sight no longer
  pollutes the inferred mask.)
- Add the derived predicate `PlayerKnowledge::knows_identity(idx) -> bool`:
  `visible_cards bit set ∨ inferred_identities[idx].is_exactly_known()`.
- Migrate every reader to one of:
  - POV-level effective `visible_cards` (for "this player can see this card"),
  - `inferred_identities[idx].is_exactly_known()` (for "this player has
    narrowed to one possibility via clues"), or
  - `knows_identity(idx)` (when both forms count).
- Tech tests that poke `visible_cards` directly: keep doing so when they
  really mean direct sight; switch to setting `inferred_identities` to a
  singleton when they mean "this player has deduced this from clues."

### Phase 3 — relocate `combined_possible_identities`

- Move `combined_possible_identities` from `PlayerKnowledge` to
  `LightweightPlayerPOV` (it now depends on the POV's effective
  `visible_cards`, not just on PK state).
- Drop the `table_state` argument from PK methods that no longer need it.

## Files in scope

| Phase | File | Change |
|-------|------|--------|
| 1 | `src/engine/knowledge/lightweight_player_pov.rs` | add POV-level `visible_cards`, `teammate_pov`, plumb through `inferred_identities` / `observable_identity_mask` |
| 1 | `src/engine/tree_action_selection_strategy.rs` | use `teammate_pov` in `best_score_at_depth` |
| 1 | `src/engine/knowledge/player_knowledge.rs` | `combined_possible_identities` consults a `visible_cards` argument instead of reading the field directly |
| 2 | `src/engine/knowledge/player_knowledge.rs` | `narrow_inferred` / `exclude_inferred` stop setting `visible_cards`; `update_with_revealed_card` stops setting `inferred_identities`; add `knows_identity` |
| 2 | `src/engine/knowledge/team_knowledge.rs` | adjust draw write-through to set only `visible_cards` |
| 2 | callers of `visible_cards` across the codebase | migrate to the appropriate of the three predicates |
| 2 | `src/engine/convention/hgroup/tech/simple_finesse.rs` (tests) | reclassify `visible_cards` pokes |
| 2 | `src/engine/convention/hgroup/tech/simple_prompt.rs` (tests) | reclassify `visible_cards` pokes |
| 2 | `src/engine/convention/hgroup/tech/direct_play_clue.rs` (tests) | reclassify `visible_cards` pokes |
| 3 | `src/engine/knowledge/player_knowledge.rs` + `lightweight_player_pov.rs` | move `combined_possible_identities` to POV |

## Risks

- Phase 2 changes the mutation semantics of `narrow_inferred` /
  `exclude_inferred`. Every site that today reads `visible_cards` to mean "I
  have deduced this card's identity" must be migrated to
  `inferred_identities[i].is_exactly_known()` or `knows_identity`.
- Tech tests poke `visible_cards` to simulate "this player has seen X"
  without going through the full draw machinery. Some of these actually meant
  "this player has deduced X" — they need to switch field. A small test
  helper (`assert_knows_identity`) may smooth the migration.
- `is_identity_known_to_holder` checks every player's `visible_cards`;
  Phase 2 must preserve its current behavior via `knows_identity`.

## Tests

- Phase 1 must make `should_clue_rank_3_instead_of_red` pass while keeping
  the full suite green.
- The companion search-regression
  (`should_not_give_a_play_clue_that_looks_like_a_save`) already passes
  (the scenario's hand composition didn't allow the omniscient lookahead to
  find a recovery) and must continue to pass.
- Add a unit test on `LightweightPlayerPOV::teammate_pov`: from Bob's POV,
  assert `teammate_pov(alice).card_identity(bob_card)` returns `None` even
  though `team_knowledge.player(alice).visible_cards` records direct sight
  of `bob_card`.
- After Phase 2, add a unit test asserting that a clue narrowing
  `inferred_identities` to a singleton does **not** set `visible_cards`,
  and conversely that `update_with_revealed_card` does **not** write into
  `inferred_identities`.

## Decision points before starting

- Phase 1: should `combined_possible_identities` take `visible_cards` as a
  new argument, or should the POV inline its own copy of the logic? Argument
  passing is the smaller diff; inlining is cleaner long-term and is what
  Phase 3 does anyway.
- Phase 2: name of the derived predicate. `knows_identity(idx)` is the
  proposal; alternatives include `identity_known(idx)` or simply leaving it
  uncalled and inlining the boolean at each site.
