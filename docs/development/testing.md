Integration tests live in `tests/`, inline unit tests use `#[cfg(test)]` blocks within source files.

Player name conventions (Alice = 0, Bob = 1, etc.) are defined in `docs/domain/hgroup.md`.

## `test-support` Feature

Several modules that build canonical game states for testing are gated on
`#[cfg(any(test, feature = "test-support"))]`. They are always compiled into test binaries but are
hidden from normal library builds. Enable the feature when writing benchmarks or any non-test binary
that needs to construct representative game states without going through JSON scenarios.

```toml
# benches/Cargo.toml or a [dev-dependencies] block
eel = { path = "..", features = ["test-support"] }
```

Or from the command line:

```bash
cargo bench --features test-support
```

### What the feature exposes

| Path | Contents |
|---|---|
| `eel::game::deck::unit_test_constant` | `NEW_DECK` (fresh no-variant `Deck`); `Deck::of(…)` constructor |
| `eel::game::deck::unit_test_constant::novariant_constants` | `NoVarCards` enum, per-card mask constants (`R1_MASK` … `P5_MASK`), `COPIES_COUNT_BY_ID` |
| `eel::game::state::table_state::unit_test_constants::no_variant_constants` | `NOVAR_5_PLAYERS_STATIC_GAME_DATA`, `initial_five_players_table_state()`, `empty_stacks_table_state(player)`, `stacked_table_state(player)` |

`initial_five_players_table_state` returns a 5-player `TableState` with empty hands and stacks —
useful as a blank canvas. `empty_stacks_table_state` and `stacked_table_state` return 3-player
states; the latter has B1–B4 already on the playing stacks.

**Scenario tests**: Game positions are stored as JSON files at `tests/scenarios/scenario{n}/table_state.json`.

Load helpers (in `tests/common/mod.rs`):

```rust
// Board state only
let (table_state, static_data) = common::load_scenario(n);

// Board state + team knowledge + history + parsed actions
let (table_state, static_data, team_knowledge, history, actions) =
    common::load_scenario_with_knowledge(n);
```

`history[i]` is the `GameStateSnapshot` before `actions[i]`, so tests can call
`tech.knowledge_updates(&actions[0], &history, &pov)` without fabricating snapshots manually.

### Scenario JSON format

```json
{
  "scenario_description": "Human-readable summary.",
  "suits": ["red", "yellow", "green", "blue", "purple"],
  "playing_stacks": [["r1"], [], [], ["b1", "b2"], []],
  "discard_pile": ["g3"],
  "clue_tokens": 5,
  "strikes": 0,
  "active_player": 0,
  "hands": [
    ["x", "x", "x", "x", "x"],
    ["r2", "b3", "y4", "b1", "r3"],
    [
      "b4",
      {"id": "g3", "positive": ["3"], "negative": ["red", "blue"], "inferred": "g3"},
      "p2", "p2", "r1"
    ]
  ],
  "prior_actions": [
    {"type": "clue", "giver": 0, "receiver": 2, "clue": "3"}
  ]
}
```

`hands` — each element is a player's hand listed slot-1-first (newest → oldest). Each slot is a plain card string (`"r3"`, `"x"`) or a full object:

| Field | Meaning |
|-------|---------|
| `id` | Card identity or `"x"` for unknown |
| `positive` | Clue values that touched this slot — narrows empathy and marks as clued |
| `negative` | Clue values that did not touch this slot — excludes those identities from empathy |
| `inferred` | Convention-based identity stored in `TeamKnowledge` (e.g. `"r3"`, `"r3b3"`) |

Clue value strings: rank digit `"1"`–`"5"` or colour name (`"red"`, `"yellow"`, `"green"`, `"blue"`, `"purple"`).

`prior_actions` — replayed at load time to build `history`. Supported types:

| Type | Fields |
|------|--------|
| `"clue"` | `giver`, `receiver` (player indices), `clue` |
| `"play"` | `player` (index), `slot` (1-indexed) |
| `"discard"` | `player` (index), `slot` (1-indexed) |

**Card ID encoding** (no-variant): `id = suit_offset + rank - 1`, offsets R=0, Y=5, G=10, B=15, P=20.
So R1=0, R5=4, Y1=5, G1=10, B1=15, P1=20, P5=24.