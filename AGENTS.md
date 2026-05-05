# Project Overview

**Eel** is a high-performance Hanabi card game bot written in Rust. It is designed around alpha-beta search (similar to
a chess engine) and uses bitmaps extensively for fast state computation.

## Common Commands

```bash
cargo build                              # Debug build
cargo build --release                    # Release build
cargo build --features test-support      # Build with test fixtures exposed (for benchmarks)
cargo test                               # Run all tests
cargo test <test_name>                   # Run a specific test by name (substring match)
cargo test --test '*'                    # Run integration tests only
cargo clippy                             # Lint
cargo fmt                                # Auto-format
```

Integration tests live in `tests/`, inline unit tests use `#[cfg(test)]` blocks within source files.

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

**Scenario tests**: Game positions are stored as JSON files at `tests/scenarios/scenario{n}/table_state.json`. Load them
in tests with `common::load_scenario(n)` (returns `(TableState, StaticGameData)`).
See `ScenarioJson` in `src/game/state/table_state_json.rs` for the schema. Cards are
encoded as strings like `"r1"`, `"b3"`, `"p4"`; unknown cards use `"x"`.

**Card ID encoding** (no-variant): `id = suit_offset + rank - 1`, where offsets are R=0, Y=5, G=10, B=15, P=20. So R1=0,
R5=4, Y1=5, G1=10, B1=15, P1=20, P4=23.

## Architecture

The codebase is split into two modules:

### `src/game/` — Pure game mechanics

- **`TableState`** (`state/table_state.rs`): Core immutable snapshot of game state used as search nodes (hands, clues,
  playing stacks, strikes).
- **`StaticGameData`**: Variant rules and player count — kept separate from `TableState` so only the changing state is
  cloned into search nodes.
- **`Deck`**: Manages card reveal state and per-card "empathy" (what is known about each card) using bitwise operations
  on `u64` bitmasks.
- **`Hand`, `PlayingStacks`, `ClueTokenBank`**: Core game entities.

### `src/engine/` — Strategy and AI

- **`KnowledgeAwareGameState`** (`knowledge_aware_game_state.rs`): WIP wrapper combining `TableState` with strategic
  knowledge. The intended integration point between game mechanics and conventions. Call `snapshot()` to capture the
  current `(TableState, TeamKnowledge)` pair as an owned `GameStateSnapshot` for history tracking.
- **`PlayerKnowledgeState`**, **`TeamKnowledge`**, **`PlayerPOV`** (`knowledge/`): Track what each player knows about
  their own hand and others'. `LightweightPlayerPOV` is the concrete borrow-based implementation; `MockPlayerPOV` (generated
  via `mockall`) is used in unit tests.
- **`ConventionSet`** (`convention/`): System for interpreting clues and selecting actions. The H-Group convention set
  lives in `convention/hgroup/`, with individual techniques as separate files in `tech/` (e.g., `critical_save`,
  `five_save`, `two_save`, `simple_prompt`, `simple_finesse`, `delayed_play_clue`).
- **`ConventionTech`**: Each technique implements three methods — `game_actions` (what to do), `matches_action` (does
  this tech explain an observed action?), and `knowledge_updates` (what to infer from this action). Concrete techs
  don't implement `ConventionTech` directly — they implement an **action-typed sub-trait** (`ClueTech`, `PlayTech`, or
  `DiscardTech` in `convention_tech.rs`) that receives pre-unwrapped typed parameters. A macro
  (`impl_convention_tech_for_clue_tech!` / `_play_tech!` / `_discard_tech!`) generates the `ConventionTech` impl,
  eliminating per-tech `if let GameAction::Clue { .. }` boilerplate.
- **H-Group priority layer** (`hgroup/h_group_tech.rs`): Clue-interpretation priority (save > play > prompt > finesse)
  is H-Group-specific, not baked into the generic layer. `HGroupClueTech: ClueTech` has a default
  `interpretation_priority = SIMPLE_PLAY_CLUE`; `SaveClueTech` and `PlayClueTech` are marker sub-traits that override
  it. A different convention system (e.g. Referential Sieve) would define its own priority scheme without touching
  `convention_tech.rs`.

## Key Design Principles

1. **Bitmaps everywhere**: `VariantCardsBitField` and `DeckCardsBitField` are `u64` bitmasks representing sets of card
   identities. Bitwise ops are used throughout for performance-critical paths.
2. **Search-first design**: `TableState` is kept minimal and cheap to clone so it can serve as alpha-beta search nodes.
   `StaticGameData` is never cloned.
3. **Convention system**: Strategic logic is separated into composable `Convention` implementations rather than
   monolithic decision code.


## H-Group Convention Terminology

| Term           | Definition                                                                                                                                                                                                        |
|----------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Chop**       | The oldest unclued card in a player's hand — the card most at risk of being discarded. Implemented by `get_chop_index` in `convention/hgroup/h_group_core.rs`.                                                    |
| **Clue focus** | The card a clue is "about". If the clue touches the chop, the chop is the focus. Otherwise, the focus is the leftmost (newest, slot 1) card that was not previously clued. Implemented by `get_clue_focus_index`. |
| **Slot**       | Slot 1 = newest card (most recently drawn). Slot N = oldest card (chop when unclued).                                                                                                                             |
| **Blind-play** | Playing a card not touched by any clue (and without narrowing empathy), relying on implicit convention-driven information.                                                                                        |

Scenario docstrings and tests refer to players by name: **Alice** = player 0 (always on turn in scenarios), **Bob** = 1,
**Cathy** = 2, **Donald** = 3, **Emily** = 4.

### Hand ordering in code

`Hand::cards()` returns cards **newest-first** (slot 1 → slot N). Consequently:

- The **leftmost** (slot 1, newest) card is the **first** element in `cards()`.
- The **chop** (oldest unclued) is the last unclued element in `cards()` — iterate with `.rev()` to find it first.

## Convention Tech Implementation

When implementing a tech's `matches_clue` method (to match an observed action), you must check if **from the clue giver's POV at the time the action was performed** the tech could have been used:

```rust
fn matches_clue(&self, player_index: PlayerIndex, touched: &[CardDeckIndex], clue: &Clue, pov: &dyn PlayerPOV) -> bool {
    // Reconstruct the clue giver's POV
    let giver_pov = pov.as_player_pov(pov.player_on_turn_index());
    // Now use giver_pov for all checks about what the clue giver knew/saw
}
```

This is critical because the observer's knowledge may differ from the clue giver's knowledge. Tests must also set up `team_knowledge` to reflect what the clue giver knows about other players' cards.

## Key Types

| Symbol                                  | Location                                         | Purpose                                                         |
|-----------------------------------------|--------------------------------------------------|-----------------------------------------------------------------|
| `TableState`                            | `src/game/state/table_state.rs`                  | Minimal game state for search nodes                             |
| `VariantCardsBitField`                  | `src/game/`                                      | Bitmask for card identity possibilities                         |
| `Deck`                                  | `src/game/`                                      | Manages empathy (knowledge) per card via bitmask arrays         |
| `ActionSelectionStrategy`               | `src/engine/`                                    | Trait for bot behavior implementations                          |
| `KnowledgeAwareGameState`               | `src/engine/knowledge_aware_game_state.rs`       | WIP: game state + strategic knowledge                           |
| `LightweightPlayerPOV`                  | `src/engine/knowledge/lightweight_player_pov.rs` | Concrete borrow-based `PlayerPOV` impl used during search       |
| `DecisionTree` / `ScoredNode`           | `src/engine/decision_tree.rs`                    | Scored candidate actions; `best_action()` picks max total score |
| `TreeActionSelectionStrategy`           | `src/engine/tree_action_selection_strategy.rs`   | Parallel root evaluation + recursive look-ahead search          |
| `Evaluator` / `DefaultEvaluator`        | `src/engine/evaluator.rs`                        | Scores leaf states during search                                |
| `ClueTech` / `PlayTech` / `DiscardTech` | `src/engine/convention/convention_tech.rs`       | Action-typed sub-traits that concrete techs implement           |
| `HGroupClueTech`                        | `src/engine/convention/hgroup/h_group_tech.rs`   | Adds `interpretation_priority` for H-Group save/play ordering   |
| `GameStateSnapshot`                     | `src/engine/game_state_snapshot.rs`              | Owned `(TableState, TeamKnowledge)` pair for history; reconstruct any player's POV via `player_pov(index, static_data)` |
| `ScenarioJson` / `build_from_scenario`  | `src/game/state/table_state_json.rs`             | Parse JSON scenario files into `(TableState, StaticGameData)`   |
