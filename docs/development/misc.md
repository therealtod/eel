## Hand ordering

`Hand::cards()` returns cards **newest-first** (slot 1 → slot N).

## Convention Tech Implementation

### `matches_clue` — use the giver's POV

When implementing `matches_clue`, reconstruct the clue giver's POV at the time the clue was given:

```rust
fn matches_clue(&self, player_index: PlayerIndex, touched: &[CardDeckIndex], clue: &Clue,
                turn: usize, history: &[GameStateSnapshot], observer_pov: &dyn PlayerPOV) -> bool {
    let giver_pov = history[turn].player_pov(observer_pov.active_player_index(), observer_pov.static_data());
    // use giver_pov for chop/focus/empathy checks
}
```

The observer's current knowledge may differ from what the giver knew when they acted. Tests must set up `team_knowledge` to reflect the giver's view.

### `clue_knowledge_updates` — called once per observer

`pov.active_player_index()` is the **observer** whose knowledge is being updated, not the clue giver. The method is called separately for each player. Typical patterns:

- **Clue receiver**: return `Hypothesis::unconditional(vec![NarrowPossibilities { focus_card, mask }])`. For a finesse (provisional), return `Hypothesis::provisional(updates, PendingTrigger::BlindPlay { ... })`.
- **Third party** (prompted/finessed): return `Hypothesis::unconditional(vec![AddSignal { card, Signal::Play { committed_identity, deadline_turn } }])`.
- **Clue giver or uninvolved observer**: return `Hypothesis::empty()`.

### `Signal` — attaching play/save/discard intent to cards

`Signal` is emitted via `KnowledgeUpdate::AddSignal` to record a convention-driven commitment:

```rust
Signal::Play {
    card_deck_index,        // which card in the deck
    committed_identity,     // the identity the convention says this card has
    deadline_turn,          // turn by which the play is expected
}
Signal::Save { slot_index, turn }
Signal::Discard { slot_index, turn }
```

`BlindPlay` (the play tech) fires on any untouched card whose `signals` contains a `Signal::Play`. So a finesse must attach a `Signal::Play` to the finessed card via `AddSignal` — that is what causes the finessed player to blind-play.

### `Hypothesis` and the two-layer knowledge model

`PlayerKnowledge` has two layers:

- **Baseline** (`inferred_identities`, `signals`): unconditional, permanent narrowings from revealed cards, confirmed hypotheses, and direct scenario setup.
- **Hypotheses** (`hypotheses`): provisional cohorts derived from interpreting observed actions. One cohort per observed action; multiple hypotheses in a cohort represent alternative interpretations.

Within a cohort, narrowings are **unioned** (the card could be any of the alternatives). Across cohorts (and against baseline), they are **intersected**.

Constructors:

```rust
Hypothesis::empty()                              // tech has nothing to claim
Hypothesis::unconditional(vec![update, ...])     // certain interpretation, baked directly into baseline
Hypothesis::provisional(updates, trigger)        // waits for confirmation before baking
```

A `PendingTrigger::BlindPlay { player, expected_card, deadline_turn }` confirms when `player` plays `expected_card`, and is rejected on any other action by that player. On confirmation, the hypothesis's updates are baked into baseline and all sibling hypotheses in the same cohort are dropped.

### Accessing effective knowledge

| Method | Returns |
|--------|---------|
| `knowledge.possible_identities(idx)` | Baseline only — does not include live hypothesis contributions |
| `knowledge.effective_inferred_mask(idx, variant)` | Baseline **and** live hypotheses — use this for decisions |
| `knowledge.has_play_signal(idx)` | True if any baseline or live hypothesis carries a `Signal::Play` |

Use `effective_inferred_mask` when you need to reason about what a player believes; `possible_identities` is rarely the right choice outside of tests.

### MCVP filter on H-Group clue techs

`HGroupClueTech` applies a **Minimum Clue Value Principle** filter by default via `clue_action_filters()`. Clue actions returned by `clue_game_actions` that fail MCVP are silently dropped before the engine sees them. If a clue tech's actions disappear unexpectedly, this filter is the first thing to check. Override `clue_action_filters()` and return `vec![]` only for techs that intentionally violate MCVP (e.g. Tempo Clue).
