# Eel

A Hanabi bot written in Rust, built around tree search and H-Group conventions.

## Overview

Eel plays [Hanabi](https://en.wikipedia.org/wiki/Hanabi_(card_game)) by combining a lookahead decision tree (similar to a chess engine) with a pluggable convention system. It uses `u64` bitmasks throughout for fast, branch-free state computation.

The library exposes a JNI interface so it can be called from a JVM host (e.g. an Android app or Kotlin backend).

## Architecture

```
src/
├── game/        # Pure game mechanics — TableState, Deck, Hand, PlayingStacks, etc.
├── engine/      # AI — search, conventions, knowledge tracking
```

### Game layer (`src/game/`)

`TableState` is a minimal, cheaply-cloneable snapshot of the game: hands, playing stacks, discard pile, clue tokens, and strikes. It is the search node. `StaticGameData` holds immutable variant rules and player count and is never cloned.

Card identities are encoded as integers: `id = suit_offset + rank − 1`, where offsets are R=0, Y=5, G=10, B=15, P=20 (no-variant). A `VariantCardsBitField` (`u64`) is a set of possible card identities; a `DeckCardsBitField` (`u64`) is a set of deck positions.

### Engine layer (`src/engine/`)

**Search** — `TreeActionSelectionStrategy` evaluates root candidates in parallel (via Rayon) and recurses `depth = number_of_players` turns deep. `DecisionTree` holds the resulting `ScoredNode` list and picks the best action.

**Conventions** — `ConventionSet` is a prioritised list of `ConventionTech` implementations. Each tech answers three questions:
- `game_actions` — what actions does this tech recommend right now?
- `matches_action` — does this tech explain an observed action?
- `knowledge_updates` — what can be inferred from that action?

**H-Group techniques implemented:**

| Technique | Description |
|---|---|
| `PlayKnownPlayable` | Play a card whose full identity is known to be playable |
| `DirectPlayClue` | Give a clue that directly identifies a playable card |
| `DelayedPlayClue` | Give a clue for a card playable after intervening plays |
| `SimplePrompt` | A clue that prompts a player to play their oldest touched card |
| `SimpleFinesse` | A clue that finesses a player's newest unclued card |
| `ColorCriticalSave` / `RankCriticalSave` | Save a critical (last-copy) card on chop |
| `FiveSave` | Save a 5 on chop with a rank-5 clue |
| `TwoSave` | Save a 2 on chop with a rank-2 clue |
| `DiscardChop` | Discard the oldest unclued card when no better action exists |

**Knowledge** — `PlayerKnowledgeState` tracks `empathy` (a bitmask array of possible identities per card), `own_hand`, and `visible_cards` for one player. `TeamKnowledge` aggregates all players. `PlayerPOVView` is the read-only view used by techs during search.

## Building

```bash
cargo build           # debug
cargo build --release # optimised
```

## Testing

```bash
cargo test                 # all tests
cargo test <name>          # filter by name substring
cargo test --test '*'      # integration tests only
cargo clippy               # lint
cargo fmt                  # format
```

Integration tests are in `tests/`. Each test file focuses on one technique (e.g. `critical_save_tests.rs`). Game positions are loaded from JSON scenario files:

```
tests/scenarios/scenario{n}/table_state.json
```

Scenarios are loaded with `common::load_scenario(n)` which returns `(TableState, StaticGameData)`. Cards in scenario files are written as `"r1"`, `"b3"`, `"p4"`, etc.; unknown cards use `"x"`.

Example scenario:

```json
{
  "suits": ["red", "yellow", "green", "blue", "purple"],
  "playing_stacks": [[], [], [], [], []],
  "discard_pile": ["p4"],
  "clue_tokens": 5,
  "strikes": 0,
  "player_on_turn": 0,
  "hands": [
    ["x", "x", "x", "x"],
    ["g5", "r2", "r2", "b2"],
    ["b1", "p3", "g4", "p4"],
    ["y3", "p5", "r1", "b4"]
  ]
}
```

## H-Group Terminology

| Term | Meaning |
|---|---|
| **Chop** | The oldest unclued card in a player's hand — most at risk of discard |
| **Clue focus** | The card a clue is "about": the chop if touched, otherwise the newest newly-touched card |
| **Slot 1** | Newest (most recently drawn) card |
| **Slot N** | Oldest card (= chop when unclued) |

`Hand::cards()` returns cards oldest-first (slot N → slot 1), so the chop is the first unclued element and slot 1 is the last element.
