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
- **`ConventionTech`**: Each technique implements `game_actions` (what to do), `matches_action` (does this tech
  explain an observed action?), and `knowledge_updates` (what to infer — returns a single `Hypothesis`). Techs that
  need to emit several mutually-exclusive sub-alternatives for the same observation (e.g. `DelayedPlayClue` fanning
  out across candidate connecting identities) additionally override `knowledge_updates_multi`, which returns a
  `HypothesisSet` (`SmallVec<[Hypothesis; 1]>`); the default impl wraps the single hypothesis so existing techs are
  unaffected. The dispatcher (`collect_hypotheses`) always invokes the multi variant. Concrete techs don't implement
  `ConventionTech` directly — they implement an **action-typed sub-trait** (`ClueTech`, `PlayTech`, or `DiscardTech`
  in `convention_tech.rs`) that receives pre-unwrapped typed parameters. A macro
  (`impl_convention_tech_for_clue_tech!` / `_play_tech!` / `_discard_tech!`) generates the `ConventionTech` impl,
  eliminating per-tech `if let GameAction::Clue { .. }` boilerplate.
- **H-Group priority layer** (`hgroup/h_group_tech.rs`): Clue-interpretation priority (save > play > prompt > finesse)
  is H-Group-specific, not baked into the generic layer. `HGroupClueTech: ClueTech` has a default
  `interpretation_priority = SIMPLE_PLAY_CLUE`; `SaveClueTech` and `PlayClueTech` are marker sub-traits that override
  it. A different convention system (e.g. Referential Sieve) would define its own priority scheme without touching
  `convention_tech.rs`.