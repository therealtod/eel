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


## Building

```bash
cargo build                         # debug
cargo build --release               # optimised
```

## Development commands

```bash
cargo test                 # all tests
cargo test <name>          # filter by name substring
cargo test --test '*'      # integration tests only (quote the glob in some shells)
cargo clippy               # lint
cargo fmt                  # format
```

## Hanabi-specific terminology

| Term           | Meaning                                                                                                                                                                                                                                                                         |
|----------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Chop**       | The slot that a player discards by default when a discard action is needed                                                                                                                                                                                                      |
| **Slot 1**     | Newest (most recently drawn) card                                                                                                                                                                                                                                               |
| **Slot N**     | Oldest card (= chop when unclued)                                                                                                                                                                                                                                               |
| **Blind-play** | The action of playing a card that has not been touched by any clue during the game (and does not have sufficient negative empathy to narrow down the card's identity). This is usually possible thanks to implicit information being communicated via convention-driven actions |
| **Alice**      | Player index 0 (always the active player in scenario descriptions)                                                                                                                                                                                                              |
| **Bob**        | Player index 1                                                                                                                                                                                                                                                                  |
| **Cathy**      | Player index 2                                                                                                                                                                                                                                                                  |
| **Donald**     | Player index 3                                                                                                                                                                                                                                                                  |
| **Emily**      | Player index 4                                                                                                                                                                                                                                                                  |


## H-Group Terminology

| Term           | Meaning                                                                                  |
|----------------|------------------------------------------------------------------------------------------|
| **Chop**       | The oldest unclued card in a player's hand — most at risk of discard                     |
| **Clue focus** | The card a clue is "about": the chop if touched, otherwise the newest newly-touched card |

