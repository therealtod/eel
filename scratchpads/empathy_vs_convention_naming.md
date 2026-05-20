# Terminology Confusion: Empathy vs. Convention-Narrowed Identities

## The Problem

In H-Group Hanabi, **empathy** has a precise meaning: the set of card identities consistent
with the public information alone — positive/negative clue touches and cards visible to the
observer. It is derivable by any informed spectator without any convention knowledge.

The codebase conflates empathy with a second, distinct concept: the **convention-narrowed
mask**, which intersects the raw empathy with inferences produced by tech hypotheses
(e.g. "this rank-3 clue is a DirectPlayClue, so the focus must be B3"). These are different
things — one is a game-rule fact, the other is a convention-derived belief.

The confusion is currently baked into the naming at two levels:

### The misleading homonym

| Site | Identifier | What it actually contains |
|---|---|---|
| `PlayerKnowledge.inferred_identities` (field) | baseline | raw clue mask + GTP — **pure game-rule empathy** |
| `PlayerKnowledge::possible_identities()` | same | accessor for the same baseline |
| `PlayerKnowledge::effective_inferred_mask()` | convention-narrowed | baseline ∩ union-of-tier-0-hypotheses |
| `PlayerPOV::inferred_identities()` (**trait method**) | convention-narrowed | routes to `combined_possible_identities` → `effective_inferred_mask` |
| `PlayerKnowledge::combined_possible_identities()` | convention-narrowed | adds observable narrowing on top of the above |

`PlayerKnowledge::inferred_identities` (the struct field) and `PlayerPOV::inferred_identities`
(the trait method) share a name but return different things. The trait method wraps the full
convention-inference pipeline; the field is the raw baseline.

### Downstream propagation

`PlayKnownPlayable.play_game_actions` calls `active_player_pov.inferred_identities(card)`,
which resolves to the convention-narrowed mask. The local variable storing the result is
named `empathy_playable`. The doc-comment on the struct reads:

> Only fires for empathy-playable cards (all remaining possibilities are playable).

This description uses "empathy" to mean "convention-narrowed" — the opposite of what the
term means in H-Group documentation and the rest of the domain model.

The same pattern repeats in test names:
`plays_empathy_playable_card`, `matches_play_false_for_empathy_playable_card_with_play_signal`.

## Why It Matters

The naming confusion caused a real diagnostic difficulty during the `should_wait_for_r2`
investigation. The diag dump printed `combined_possible_identities` under the label `empathy`,
showing only B3. It looked like a game-rule fact ("empathy says B3"), but it was actually a
convention inference from `DirectPlayClue`'s tier-0 hypothesis. The raw empathy — {R3, Y3,
G3, B3} — was sitting in the `inferred` column of the same dump, unnoticed.

More broadly: any future bug where a convention inference incorrectly collapses the identity
space will look identical to a raw-clue narrowing failure if the two layers are not clearly
named. The distinction matters for debugging and for ensuring new techs don't accidentally
bake inferences into the baseline.

## Proposed Rename

### `PlayerKnowledge` struct

| Current | Proposed |
|---|---|
| `.inferred_identities` (field) | `.baseline_identities` |
| `narrow_inferred(idx, mask, …)` | `narrow_baseline(idx, mask, …)` |
| `exclude_inferred(idx, mask, …)` | `exclude_baseline(idx, mask, …)` |
| `possible_identities(idx)` | `baseline_identities(idx)` (or keep as `raw_identities`) |

### `PlayerPOV` trait method

| Current | Proposed |
|---|---|
| `inferred_identities(idx)` | `effective_identities(idx)` |

This name conveys that the result is the fully-resolved view the player acts on, combining
game-rule facts with live convention reasoning — without claiming it is the empathy.

### `PlayKnownPlayable`

| Current | Proposed |
|---|---|
| `empathy_playable` (local var) | `convention_playable` |
| doc: "empathy-playable cards" | "convention-known-playable cards" |
| test names `*_empathy_playable_*` | `*_convention_playable_*` |

### Diag script

The `empathy=` label in `scripts/diag.sh` (and the generated `tests/_diag.rs`) should print
two separate lines:
- `baseline=` — `possible_identities()` (raw game-rule mask)
- `effective=` — `combined_possible_identities()` (convention-narrowed)

## Risk and Scope

This is a pure rename with no semantic change. The main risk is mechanical breakage from
missed call-sites. The affected surface:

- `PlayerKnowledge` field read/written everywhere team knowledge is updated (~30 sites in
  `knowledge_aware_game_state.rs`, `player_knowledge.rs`, tech `knowledge_updates` methods)
- `PlayerPOV::inferred_identities` implemented in `LightweightPlayerPOV` and called by every
  tech that evaluates card possibilities (~15 call-sites across hgroup techs, evaluator, etc.)
- Test names in `play_known_playable.rs` (4 tests)
- The diag script and generated diagnostic test

No logic changes are required. A `grep` for `inferred_identities` and `empathy` across `src/`
and `tests/` gives the full affected set before starting.
