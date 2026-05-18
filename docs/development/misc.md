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
- **Third party** (prompted/finessed): return `Hypothesis::unconditional(vec![AddSignal { card, Signal::Play { committed_identity } }])`.
- **Clue giver or uninvolved observer**: return `Hypothesis::empty()`.

### `clue_knowledge_updates_multi` — emit multiple sibling hypotheses

A tech that needs to emit **several mutually-exclusive sub-alternatives** for the same observed action (rather than a single union-mask hypothesis) overrides `clue_knowledge_updates_multi`, which returns a `HypothesisSet = SmallVec<[Hypothesis; 1]>`. The default implementation wraps the single hypothesis returned by `clue_knowledge_updates`, so existing techs need no change.

The sibling hypotheses should share an `alt_group` so that confirming one prunes only the others in that group, leaving cohort-mate hypotheses from sibling techs untouched. See the `alt_group` section below.

Canonical example: `DelayedPlayClue` emits one provisional sub-hypothesis per candidate connecting identity when the connecting card's empathy is ambiguous-known-playable (e.g. a touched "1 of unknown color"). Each sub-hypothesis pins the focus to its matching `connecting_id + 1` and carries an identity-keyed `BlindPlay` trigger, all sharing `alt_group = connecting_card_idx`. When the connecting card plays as a specific identity, the matching sub-hypothesis confirms and the others reject — leaving DirectPlayClue's interpretation (in the same cohort, `alt_group = None`) intact so the focus mask becomes `{confirmed_connector_id + 1} ∪ {direct candidates}`.

The dispatcher (`collect_hypotheses`) calls `knowledge_updates_multi`, so what a tech emits there is what the engine sees. The single-hypothesis `clue_knowledge_updates` remains useful for tests that pattern-match `updates.immediate` / `updates.trigger` and for techs that legitimately have only one interpretation per action.

### `Signal` — attaching play/save/discard intent to cards

`Signal` is emitted via `KnowledgeUpdate::AddSignal` to record a convention-driven commitment:

```rust
Signal::Play {
    card_deck_index,        // which card in the deck
    committed_identity,     // the identity the convention says this card has
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
Hypothesis::provisional(updates, trigger)        // waits for confirmation before baking; alt_group = None
Hypothesis::provisional_grouped(updates, trigger, alt_group)
                                                 // provisional with selective rejection scope
```

`PendingTrigger::BlindPlay { player, expected_card, expected_identity }` confirms when `player`'s next action plays `expected_card` — and, if `expected_identity = Some(id)`, the revealed identity also matches `id`. A mismatch on either deck index or expected identity (when set) rejects the hypothesis; any non-play action by `player` rejects too. `expected_identity = None` keeps the legacy "any play of `expected_card` confirms" behavior used by SimpleFinesse.

The played card's resolved identity is threaded into `resolve_pending` by `apply_play` / replay reconstruction so identity-keyed triggers can compare against it.

### `alt_group` — scoping confirmation's effect on cohort siblings

A `Hypothesis` carries an optional `alt_group: Option<HypothesisId>` that controls **which sibling hypotheses get pruned when this one confirms**:

- `alt_group = None` (legacy cohort-wide drop). On confirmation:
  - the hypothesis's `immediate` updates are baked into baseline,
  - **all** sibling hypotheses sharing the same `cohort_id` are dropped.

  Used by SimpleFinesse: a blind-play confirms the finesse interpretation outright, refuting every other interpretation (DirectPlay, CriticalSave, etc.) of the same clue.

- `alt_group = Some(group)` (scoped drop). On confirmation:
  - the hypothesis is **kept** in the cohort with its trigger cleared (so its mask continues to contribute to the cohort union),
  - it is **not** baked into baseline (baking a narrow mask would intersect against unconditional siblings outside the group and collapse the focus to ∅),
  - only sibling hypotheses with matching `(cohort_id, alt_group)` are dropped.

  Used by DelayedPlayClue's per-connecting-id sub-hypotheses: confirming "the connecting card played as R1" must drop "delayed via Y1" and "delayed via G1" (same `alt_group = connecting_card_idx`), but must leave DirectPlayClue's hypothesis (no `alt_group`) in the cohort so its `{B2, P2}` mask survives.

Rejection always drops just the rejected hypothesis (`alt_group` plays no role in rejection).

### Accessing effective knowledge

| Method | Returns |
|--------|---------|
| `knowledge.possible_identities(idx)` | Baseline only — does not include live hypothesis contributions |
| `knowledge.effective_inferred_mask(idx, variant)` | Baseline **and** live hypotheses — use this for decisions |
| `knowledge.has_play_signal(idx)` | True if any baseline or live hypothesis carries a `Signal::Play` |

Use `effective_inferred_mask` when you need to reason about what a player believes; `possible_identities` is rarely the right choice outside of tests.

### MCVP filter on H-Group clue techs

`HGroupClueTech` applies a **Minimum Clue Value Principle** filter by default via `clue_action_filters()`. Clue actions returned by `clue_game_actions` that fail MCVP are silently dropped before the engine sees them. If a clue tech's actions disappear unexpectedly, this filter is the first thing to check. Override `clue_action_filters()` and return `vec![]` only for techs that intentionally violate MCVP (e.g. Tempo Clue).
