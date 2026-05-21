# Eel

A Hanabi bot written in Rust, built around tree search and H-Group conventions.

## Overview

Eel plays [Hanabi](https://en.wikipedia.org/wiki/Hanabi_(card_game)) by combining a lookahead decision tree (similar to
a chess engine) with a pluggable convention system. It uses `u64` bitmasks throughout for fast, branch-free state
computation.

## Architecture

```
src/
├── game/        # Pure game mechanics — TableState, Deck, Hand, PlayingStacks, etc.
├── engine/      # AI — search, conventions, knowledge tracking
```

See the subsections below for internal design details.

### Game layer (`src/game/`)

`TableState` is a minimal, cheaply-cloneable snapshot of the game: hands, playing stacks, discard pile, clue tokens, and
strikes. It is the search node. `StaticGameData` holds immutable variant rules and player count and is never cloned.

Card identities are encoded as integers: `id = suit_offset + rank - 1`, where offsets are R=0, Y=5, G=10, B=15, P=20 (
no-variant). A `VariantCardsBitField` (`u64`) is a set of possible card identities; a `DeckCardsBitField` (`u64`) is a
set of deck positions.

### Engine layer (`src/engine/`)

**Search** — `TreeActionSelectionStrategy` evaluates root candidates in parallel (via Rayon) and recurses
`depth = number_of_players` turns deep. `DecisionTree` holds the resulting `ScoredNode` list and picks the best action.

**Conventions** — `ConventionSet` is a prioritised list of `ConventionTech` implementations. Each tech answers three
questions:

- `game_actions` — what actions does this tech recommend right now?
- `matches_action` — does this tech explain an observed action?
- `knowledge_updates` — what can be inferred from that action?

**Knowledge** — `PlayerKnowledgeState` tracks `empathy` (a bitmask array of possible identities per card), `own_hand`,
and `visible_cards` for one player. `TeamKnowledge` aggregates all players. `LightweightPlayerPOV` is the read-only view used
by techs during search.


#### Convention tech trait hierarchy

`ConventionTech` is the top-level trait consumed by the engine. Concrete techniques do not implement it directly;
instead they implement one of three **action-typed sub-traits** in `convention_tech.rs`, which receive pre-unwrapped
typed parameters instead of a raw `&GameAction`:

```
ConventionTech          (engine-facing: &GameAction)
├── ClueTech            (clue-specific: player_index, &[CardDeckIndex], &Clue)
├── PlayTech            (play-specific: player_index, CardDeckIndex)
└── DiscardTech         (discard-specific: player_index, CardDeckIndex)
```

A macro (`impl_convention_tech_for_clue_tech!` / `_play_tech!` / `_discard_tech!`) generates the `ConventionTech`
impl for each concrete type, including the `if let GameAction::Clue { .. }` guard that would otherwise be duplicated
in every technique.

#### H-Group clue priority layer

Priority ordering between clue techniques (save > play clue > prompt > finesse) is an H-Group-specific concept and
lives in `hgroup/h_group_tech.rs`, not in the generic layer. `HGroupClueTech` extends `ClueTech` with a default
`interpretation_priority` of `SIMPLE_PLAY_CLUE`. Two marker sub-traits override it:

```
HGroupClueTech: ClueTech    (default priority = SIMPLE_PLAY_CLUE = 1)
├── SaveClueTech             (priority = SAVE = 0)
└── PlayClueTech             (inherits SIMPLE_PLAY_CLUE; SimplePrompt/SimpleFinesse override to PROMPT=2/FINESSE=3)
```

A different convention system (e.g. Referential Sieve) would define its own priority scheme without touching
`convention_tech.rs`.

**H-Group techniques implemented:**

| Technique                                | Description                                                    |
|------------------------------------------|----------------------------------------------------------------|
| `PlayKnownPlayable`                      | Play a card whose full identity is known to be playable        |
| `DirectPlayClue`                         | Give a clue that directly identifies a playable card           |
| `DelayedPlayClue`                        | Give a clue for a card playable after intervening plays        |
| `SimplePrompt`                           | A clue that prompts a player to play their oldest touched card |
| `SimpleFinesse`                          | A clue that finesses a player's newest unclued card            |
| `ColorCriticalSave` / `RankCriticalSave` | Save a critical (last-copy) card on chop                       |
| `FiveSave`                               | Save a 5 on chop with a rank-5 clue                            |
| `TwoSave`                                | Save a 2 on chop with a rank-2 clue                            |
| `DiscardChop`                            | Discard the oldest unclued card when no better action exists   |
| `BlindPlay`                              | Play an untouched card that carries a convention-issued `Signal::Play` |


## Building

```bash
cargo build                         # debug
cargo build --release               # optimised
```

## Docker

### Build the image

```bash
docker build -t eel .
```

### Run

```bash
docker run eel --name <name> [--count <count>]
```

### Run with logging

```bash
docker run -e RUST_LOG=eel::search=debug eel --name <name>
```

### Run tests in a container

```bash
docker build -t eel-builder --target builder .
docker run --rm eel-builder cargo test
```

## Debugging the decision tree

`TreeActionSelectionStrategy::scored_actions()` returns `Vec<ScoredNode>` sorted best-first. Each `ScoredNode` implements `Display` and shows the total score, principal variation (the full action line), and the leaf-state breakdown:

```rust
let nodes = strategy.scored_actions(&pov, &conventions);
for (i, node) in nodes.iter().enumerate() {
    println!("=== Candidate {} ===\n{}", i + 1, node);
}
```

You can also emit structured logs via the `eel::search` tracing target:

```bash
RUST_LOG=eel::search=debug cargo test <name> -- --nocapture  # per-candidate summaries
RUST_LOG=eel::search=trace cargo test <name> -- --nocapture  # also per-ply detail
```

## Development commands

```bash
cargo test                 # all tests
cargo test <name>          # filter by name substring
cargo test --test '*'      # integration tests only (quote the glob in some shells)
cargo clippy               # lint
cargo fmt                  # format
```

## Test scenarios

Game positions are stored as JSON files under `tests/scenarios/`. Load them via helpers in `tests/common/mod.rs`:

```rust
// Board state only (numeric scenario)
let (table_state, static_data) = common::load_scenario("simple_finesse", 1);

// Board state only (semantic path — used by search_regression.rs)
let (table_state, static_data) = common::load_scenario_by_name("search/my_scenario");

// Board state + team knowledge + history + parsed actions (numeric scenario)
let (table_state, static_data, team_knowledge, history, actions) =
    common::load_scenario_with_knowledge("simple_finesse", 2);

// Board state + team knowledge (semantic path)
let (table_state, static_data, team_knowledge, history, actions) =
    common::load_scenario_by_name_with_knowledge("search/my_scenario");
```

`history[i]` is the `GameStateSnapshot` taken before `actions[i]`, so tests can call
`tech.knowledge_updates(&actions[0], &history, &pov)` without manually fabricating snapshots
or hardcoding action structs.

### Scenario JSON format

```json
{
  "scenario_description": "Human-readable summary of the position.",
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
      "p2",
      "p2",
      "r1"
    ]
  ],
  "prior_actions": [
    {"type": "clue", "giver": 0, "receiver": 2, "clue": "3"}
  ]
}
```

**`hands`** — Each element is a player's hand listed slot-1-first (newest → oldest).
Each slot is either a plain card string or a full object:

| Field      | Type             | Meaning                                                                                             |
|------------|------------------|-----------------------------------------------------------------------------------------------------|
| `id`       | `string`         | Card identity (`"r3"`, `"b4"`, …) or `"x"` for an unknown card.                                   |
| `positive` | `string[]`       | Clue values that **touched** this slot. Each entry narrows the deck empathy and marks the slot as clued. |
| `negative` | `string[]`       | Clue values that were given but did **not** touch this slot. Each entry excludes those identities from the empathy mask. |
| `inferred` | `string \| null` | Convention-based identity, stored in `TeamKnowledge`. Same format as `id` but may concatenate multiple possibilities (`"r3b3"`). |

Clue value strings: rank digit `"1"`–`"5"` or colour name `"red"`, `"yellow"`, `"green"`,
`"blue"`, `"purple"`. Using `positive`/`negative` per slot replaces the old separate
`empathy`, `clued_cards`, and `inferred_identities` top-level arrays; plain string slots
remain fully supported for positions with no clue history.

**`prior_actions`** — Actions that happen after the scenario state, in order. Supported types:

| Type      | Fields                                          | Notes                                                         |
|-----------|-------------------------------------------------|---------------------------------------------------------------|
| `"clue"`  | `giver`, `receiver` (player indices), `clue`    | Touched cards computed automatically from hand identities.    |
| `"play"`  | `player` (player index), `slot` (1-indexed)     |                                                               |
| `"discard"` | `player` (player index), `slot` (1-indexed)   |                                                               |

The loader replays `prior_actions` at the table-state level to build `history` and returns the
parsed `Vec<GameAction>`, so neither needs to be constructed in test code.

### Card ID encoding (no-variant)

`id = suit_offset + rank − 1`, with suit offsets R=0, Y=5, G=10, B=15, P=20.

| Card | R1 | R2 | R3 | R4 | R5 | Y1 | … | G1 | … | B1 | … | P1 | … | P5 |
|------|----|----|----|----|----|----|---|----|----|----|----|----|----|-----|
| ID   | 0  | 1  | 2  | 3  | 4  | 5  | … | 10 | … | 15 | … | 20 | … | 24 |

## Hanabi-specific terminology

| Term                 | Meaning                                                                                                                                                                                                                                                                         |
|----------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Chop**             | The slot that a player discards by default when a discard action is needed                                                                                                                                                                                                      |
| **Slot 1**           | Newest (most recently drawn) card                                                                                                                                                                                                                                               |
| **Slot N**           | Oldest card (= chop when unclued)                                                                                                                                                                                                                                               |
| **Blind-play**       | The action of playing a card that has not been touched by any clue during the game (and does not have sufficient negative empathy to narrow down the card's identity). This is usually possible thanks to implicit information being communicated via convention-driven actions |
| **Bottom Deck Risk** | A critical card that, in case it's the bottom of the deck, will prevent the team from achieving max score. Usually abbreviated with 'BDR'                                                                                                                                       |
| **Empathy**          | Set of possible card identities that a hand slot can contain based purely on game rules                                                                                                                                                                                         |
| **Alice**            | Player index 0 (always the active player in scenario descriptions)                                                                                                                                                                                                              |
| **Bob**              | Player index 1                                                                                                                                                                                                                                                                  |
| **Cathy**            | Player index 2                                                                                                                                                                                                                                                                  |
| **Donald**           | Player index 3                                                                                                                                                                                                                                                                  |
| **Emily**            | Player index 4                                                                                                                                                                                                                                                                  |


## H-Group Terminology

| Term           | Meaning                                                                                  |
|----------------|------------------------------------------------------------------------------------------|
| **Chop**       | The oldest **unclued** card — more precise than the general Hanabi sense above           |
| **Clue focus** | The card a clue is "about": the chop if touched, otherwise the newest newly-touched card |

