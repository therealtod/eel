Integration tests live in `tests/`, inline unit tests use `#[cfg(test)]` blocks within source files.

## Test organisation

| File | Purpose |
|------|---------|
| `tests/simple_finesse_tests.rs` | Per-tech tests for `SimpleFinesse` — generation, knowledge updates, hypothesis resolution |
| `tests/simple_prompt_tests.rs` | Per-tech tests for `SimplePrompt` — generation and knowledge-update semantics |
| `tests/search_regression.rs` | Engine non-regression suite — verifies the full search stack selects the correct best action on known positions |
| `tests/replay_regression.rs` | Replay-based regression suite — steps a hanab.live JSON replay to a specific turn and asserts the engine's recommendation. See `tests/replays/` for the corpus. |

### When to use a replay test vs. a search-regression scenario

Use a **replay test** when the correct action at turn N depends on decisions that happened in earlier turns (e.g. which clue was given three turns ago changes what the engine should do now). The replay runner reconstructs the full game history, so convention state accumulated over prior actions is preserved.

Use a **search-regression scenario** (JSON in `tests/scenarios/search/`) when the position itself is sufficient to determine the correct action. Scenarios are self-contained and fast to create; they are the preferred format whenever history is not load-bearing.

### Per-tech tests

Each file exercises one convention tech in isolation: `game_actions` generation, `knowledge_updates`, and hypothesis resolution. Scenarios live under `tests/scenarios/{tech}/{n}/table_state.json` (numeric IDs).

### Search regression suite

`tests/search_regression.rs` holds positions where the correct best action is known in advance. The test runs the full `TreeActionSelectionStrategy` (minimax search + `DefaultEvaluator`) with the complete `HGroupConventionSet` and asserts the top-ranked action. Scenarios live under `tests/scenarios/search/{name}/table_state.json` (semantic folder names).

Each scenario JSON needs only a board position (`playing_stacks`, `hands`, `clue_tokens`, etc.) — no `prior_actions` are required unless history is needed. The expected action is declared as a Rust pattern-match assertion in the test function.

**Adding a new search regression scenario:**
1. Create `tests/scenarios/search/{descriptive_name}/table_state.json` describing the position.
2. Add a `#[test]` function in `tests/search_regression.rs` that calls `search_best_action("{descriptive_name}")` and pattern-matches the result.
3. Choose positions where the correct action is unambiguous enough that any correct implementation of the full convention set must agree.

Player name conventions (Alice = 0, Bob = 1, etc.) are defined in `docs/domain/hgroup.md`.

### Snapshot extractor

`src/bin/snapshot.rs` extracts a self-contained scenario JSON from a hanab.live replay at a given turn:

```bash
cargo run --release --bin snapshot -- \
    --replay logs/14/game_0042.json \
    --turn 22 \
    --out tests/scenarios/search/slowplay_at_22/table_state.json \
    --description "Bot slowplayed g3 instead of saving b5 on chop"
```

The tool prints the engine recommendation at that turn and a suggested test stub for `search_regression.rs`. It also compares the engine recommendation before and after serialisation — if they differ, the scenario format cannot faithfully represent the convention state accumulated over prior turns, and the tool warns you to use a replay regression test instead.

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

**Scenario tests**: Game positions are stored as JSON files at `tests/scenarios/{tech}/{n}/table_state.json`.
Each tech folder can contain multiple numbered scenarios.

Load helpers (in `tests/common/mod.rs`):

```rust
// Board state only (numeric scenario)
let (table_state, static_data) = common::load_scenario("simple_finesse", 1);

// Board state + team knowledge + history + parsed actions (numeric scenario)
let (table_state, static_data, team_knowledge, history, actions) =
    common::load_scenario_with_knowledge("simple_finesse", 2);

// Board state only (semantic path — used by search_regression.rs)
let (table_state, static_data) = common::load_scenario_by_name("search/play_known_playable");

// Board state + team knowledge (semantic path)
let (table_state, static_data, team_knowledge, history, actions) =
    common::load_scenario_by_name_with_knowledge("search/direct_play_clue_is_top_choice");
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