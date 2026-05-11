## Key Design Principles

1. **Bitmaps everywhere**: `VariantCardsBitField` and `DeckCardsBitField` are `u64` bitmasks representing sets of card
   identities. Bitwise ops are used throughout for performance-critical paths.
2. **Search-first design**: `TableState` is kept minimal and cheap to clone so it can serve as alpha-beta search nodes.
   `StaticGameData` is never cloned.
3. **Convention system**: Strategic logic is separated into composable `Convention` implementations rather than
   monolithic decision code.

