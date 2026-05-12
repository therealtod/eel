## Hand ordering

`Hand::cards()` returns cards **newest-first** (slot 1 → slot N).

## Convention Tech Implementation

### `matches_clue` — existential over observer-empathy

`matches_clue` must answer "from the observer's POV, is there any focus/chop identity consistent
with what *they* know that would have constituted this tech from the actor's POV?". Reading
`giver_pov.card_identity(focus)` is wrong from the receiver's perspective: the receiver cannot
see her own focus card, so the actor's identification of it is information she does not have.

The idiom used uniformly across techs:

```rust
fn matches_clue(&self, player_index: PlayerIndex, touched: &[CardDeckIndex], clue: &Clue,
                turn: usize, history: &[GameStateSnapshot], observer_pov: &dyn PlayerPOV) -> bool {
    let Some(snap) = history.get(turn) else { return false; };
    let giver = snap.table_state.active_player_index;
    let giver_pov = snap.player_pov(giver, observer_pov.static_data());
    let Some(focus) = get_clue_focus(player_index, touched, &giver_pov) else { return false; };

    let static_data = observer_pov.static_data();
    let total_ids = static_data.variant.number_of_suits as usize * static_data.variant.stacks_size as usize;
    let clue_mask = static_data.variant.empathy_for_clue(clue).as_bits();
    let candidates = observer_pov.empathy(focus).as_bits() & clue_mask;
    (0..total_ids).any(|id| (candidates & (1u64 << id)) != 0 && tech_predicate(id, &giver_pov))
}
```

- For non-receiver observers `observer_pov.empathy(focus)` is a singleton, so the existential
  collapses to the actor's direct check — identical to the old behaviour.
- For the receiver, `observer_pov.empathy(focus)` is wide; the existential captures her genuine
  ambiguity. `tech_predicate(id, &giver_pov)` evaluates the structural conditions (away_value,
  finesse position of a teammate, criticality, …) over teammates' hands, which both observer
  and giver can see, so reading from `giver_pov` does not leak information.

Save techs use the same shape but iterate over the chop card and check against
`empathy_for_clue` of the actual clue given.

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
