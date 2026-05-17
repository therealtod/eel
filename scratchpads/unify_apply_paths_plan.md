# Unify replay and search `apply` paths

## Motivation

Today there are two separate code paths that apply a `Play` or `Discard` action and
update knowledge:

1. **`KnowledgeAwareGameState::apply`** (`src/engine/knowledge_aware_game_state.rs`) — used
   by the search. It must reason under hidden information: empathy collapsing, phantom
   plays, truth-vs-empathy reconciliation against the searcher's POV.
2. **`ReplayRunner::apply_play` / `apply_discard`** (`src/engine/replay/reconstruct.rs`) —
   used by replay. It has the ground-truth deck, so it can short-circuit identity
   resolution and call `update_with_play_action_of_specific_card` directly.

Each path independently re-implements the surrounding *orchestration*:

- collect hypotheses (`collect_hypotheses`)
- apply the play/discard effect on `TableState` + own-hand
- draw replacement card
- store the actor's cohort on each target's knowledge (`apply_cohort`, per-target hand
  filtering)
- resolve pending triggers across all players (`resolve_pending`)

This duplication is the bug surface: `ReplayRunner::apply_play` and `apply_discard`
forgot the `resolve_pending` sweep, so `BlindPlay` triggers from clues were never
resolved on subsequent plays/discards. A SimpleFinesse tier-1 hypothesis sat alive
forever, the tier-0 sibling was never pruned, and the engine misread a clued y2 as
trash. The fix landed in commit *(pending)* as a local patch (added `resolve_pending`
calls to both runner methods), but the underlying drift hazard remains.

The goal is to make `KnowledgeAwareGameState::apply` the single owner of orchestration
and let it dispatch only the **identity-resolution** step to a small trait. Replay
provides a "ground-truth" resolver; search provides an "empathy + truth-POV" resolver.

## What stays orthogonal vs. what differs

| Step                                              | Search                                   | Replay                                  | Shared? |
|---|---|---|---|
| Build `GameAction` & turn counter                 | same                                     | same                                    | **shared** |
| `collect_hypotheses` for the actor                | same                                     | same (currently passes `&[]` history)   | **shared** |
| Resolve played card identity → `card_id` or none  | empathy + signal + `truth.card_identity` | `actual_deck[card_deck_index]`          | **differs** |
| Apply play to `TableState`                        | `update_with_play_action_of_specific_card` (singleton id) / `update_with_play_action` (phantom or unknown) | always `update_with_play_action_of_specific_card` | **differs (a function of resolver output)** |
| `add_phantom_play` increment                      | maybe                                    | never                                   | **differs (resolver-driven)** |
| `remove_card_from_own_hand` + draw next card      | search: `update_with_unkown_card_draw`   | replay: `draw_next_card` (specific card)| **differs in draw** |
| Apply per-target cohort with own-hand filtering   | same                                     | same                                    | **shared** |
| `resolve_pending` sweep for all players           | done                                     | **missed** (the bug)                    | **shared** |

So the only true variation between modes is:

1. How the played card's identity is determined (and whether it triggers a phantom play).
2. How the replacement draw is chosen (search synthesizes via `update_with_unkown_card_draw`; replay pulls the next deck index).

Everything else can be — and should be — written once.

## Proposed design

### 1. Introduce a `PlayResolver` trait

```rust
// src/engine/play_resolver.rs (new)

pub enum ResolvedPlay {
    /// The card's identity is known. Caller will call
    /// `update_with_play_action_of_specific_card(card_deck_index, card_id)`.
    Known(VariantCardId),
    /// The card is known-playable but ambiguous between several identities
    /// (search-only). Caller will call `update_with_play_action` and
    /// `add_phantom_play`.
    KnownPlayableAmbiguous,
    /// No identity information (hidden-info fallback). Caller will call
    /// `update_with_play_action` without scoring.
    Unknown,
}

pub trait PlayResolver {
    fn resolve_play(
        &self,
        player_index: PlayerIndex,
        card_deck_index: CardDeckIndex,
        state: &KnowledgeAwareGameState,
    ) -> ResolvedPlay;

    /// Identity for a discarded card — replay knows it from the deck;
    /// search can use the truth POV (same source it already uses for `truth`).
    fn resolve_discard(
        &self,
        player_index: PlayerIndex,
        card_deck_index: CardDeckIndex,
        state: &KnowledgeAwareGameState,
    ) -> VariantCardId;

    /// What card index is drawn into the actor's hand after a play/discard.
    /// `None` means "synthesize a fresh hidden-info draw" (search default).
    fn draw_next(&mut self, player_index: PlayerIndex) -> Option<CardDeckIndex>;
}
```

Two implementations:

- `TruthPovResolver<'a>` (search): wraps the existing `truth: &dyn PlayerPOV`. Its
  `resolve_play` runs the current empathy → `truth.card_identity` reconciliation logic
  from `apply_play` (lines 333–388 in `knowledge_aware_game_state.rs`). `draw_next`
  returns `None`.
- `GroundTruthResolver<'a>` (replay): wraps `&[VariantCardId]` (the actual deck) and a
  mutable draw cursor. `resolve_play` returns `Known(actual_deck[card_deck_index])`.
  `draw_next` returns `Some(next_deck_index)` and advances the cursor.

### 2. Make `KnowledgeAwareGameState::apply` accept a resolver

Replace the current `truth: &dyn PlayerPOV` parameter with `resolver: &mut dyn PlayResolver`. The truth-aware logic moves *into* `TruthPovResolver`.

Pseudocode for the unified play path:

```rust
fn apply_play(&mut self, cdi, conv, resolver) {
    let p = self.table_state.active_player_index();
    let action = GameAction::Play { player_index: p, card_deck_index: cdi, turn: self.table_state.current_turn };

    let actor_hypotheses = collect_hypotheses(conv.techs(), &action,
                                              &self.history,
                                              &self.player_pov(p));

    match resolver.resolve_play(p, cdi, self) {
        ResolvedPlay::Known(card_id) => self.table_state.update_with_play_action_of_specific_card(cdi, card_id, &self.static_data),
        ResolvedPlay::KnownPlayableAmbiguous => { self.table_state.update_with_play_action(cdi); self.add_phantom_play(); }
        ResolvedPlay::Unknown => self.table_state.update_with_play_action(cdi),
    }

    self.remove_card_from_own_hand(p, cdi);
    match resolver.draw_next(p) {
        Some(next) => { self.table_state.update_with_draw_action(next); self.team_knowledge.player_mut(p).own_hand |= 1u64 << next; }
        None       => self.update_with_unkown_card_draw(p),
    }

    self.apply_actor_cohort(p, actor_hypotheses);  // existing per-target filter+apply_cohort, factored out
    // resolve_pending runs unconditionally at the end of `apply`, as today.
}
```

### 3. Migrate the replay runner

Delete `ReplayRunner::apply_play` and `apply_discard`. Replace the runner's `step()` body
with a single `self.game.apply(&game_action, self.convention_set, &mut resolver)` call,
where `resolver` is a `GroundTruthResolver` borrowing `&self.actual_deck` and holding
the draw cursor. The Clue branch becomes uniform with Play/Discard (it already calls
`game.apply`; just thread the resolver).

### 4. Migrate search call sites

Each existing `state.apply(&action, conv, &truth_pov)` becomes
`state.apply(&action, conv, &mut TruthPovResolver::new(&truth_pov))`. The wrapper is
cheap (a single reference field) and stack-allocated.

## Migration steps (small, mergeable PRs)

1. **Extract orchestration helpers.** Pull the per-target cohort filtering + `apply_cohort`
   sequence out of both `apply_play`s into a private `KnowledgeAwareGameState::apply_actor_cohort`.
   No behavior change. Verifies the steps really are shared.
2. **Introduce `PlayResolver` + `TruthPovResolver`.** Refactor `apply_play` and
   `apply_discard` in `KnowledgeAwareGameState` to call the resolver; leave the runner
   alone. All existing search tests must pass.
3. **Introduce `GroundTruthResolver` and rewrite `ReplayRunner::step`.** Delete the
   duplicated runner methods. Run replay regression + selfplay tests.
4. **Cleanup.** Remove `truth: &dyn PlayerPOV` argument from `apply` (now lives inside
   `TruthPovResolver`), drop the now-unused `LightweightPlayerPOV` clone boilerplate at
   the runner's Clue call-site.

## Risks & considerations

- **Hot-path allocation.** `TruthPovResolver` and `GroundTruthResolver` must be zero-alloc
  on construction (no `Box`, no `Vec`). The search clones state per recursive call;
  building a wrapper per call is fine, but the resolver should not own any heap data.
- **`PlayResolver::resolve_play` borrows `&KnowledgeAwareGameState`.** That borrow ends
  before we mutate `self`, so no borrow conflict. (Same pattern the current
  `apply_play` uses with its `truth` argument.)
- **Cohort generation in the runner currently passes `&[]` history.** That's a latent
  inconsistency with the search path (which passes `&self.history`). Worth aligning in
  step 1 — likely just a behavior improvement (techs that consult history will work in
  replay too), but verify with the replay regression suite.
- **Tests that construct `KnowledgeAwareGameState` directly** and call `apply` will need
  the trivial signature update. There are ~a dozen call sites; they all live in
  `src/engine/**/tests` and the integration tests.
- **No change to the convention layer.** `Hypothesis`, `PendingTrigger`, `resolve_pending`
  are untouched. This is purely a dispatch refactor.

## Out of scope

- Make/unmake undo for the search hot path (`scratchpads/search_improvements_plan.md`
  item 2).
- Any change to how `Draw` actions interact with knowledge.
- Refactoring `apply_clue` (already shared by both modes via `game.apply`).
