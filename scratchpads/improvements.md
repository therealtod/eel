Tier 1 — highest impact

1. Search-node clone copies StaticGameData and the whole game history. KnowledgeAwareGameState derives Clone and is the unit cloned per-ply in
   search (tree_action_selection_strategy.rs:548, plus :899, :1110, :845, :1372). Beyond the intended cheap table_state/team_knowledge, it owns:
- static_data: StaticGameData (:111) — directly violates AGENTS.md ("StaticGameData should not be cloned into search nodes").
- history: Vec<GameStateSnapshot> (:117) — each snapshot is a full TableState + TeamKnowledge, so every per-ply clone deep-copies the entire
  accumulated game history.

Net effect: search-node clone cost scales with game length × player count — the exact coupling the TableState/StaticGameData split was designed
to prevent, just not carried through to the wrapper. With ~60–100 clones per root eval, plausibly a dominant cost. I'd benchmark first (per
style.md), then split the cloned "search node" from the borrowed "session context" (static_data, history). Notably, apply never appends to
history during rollout, so the search can likely borrow it instead of cloning.

Tier 2 — structural

2. h_group_core.rs is a 2779-line god-module spanning ~5 domains (hand position, clue generation, MCVP, GTP, Save principle) — a clean module
   split candidate.
3. KnowledgeAwareGameState::apply_clue (:569–799) mixes orchestration with ~230 lines of inline convention dispatch and a TableState clone
   inside the per-observer loop (:747) just to set one usize.
4. candidate_actions_with_provenance (:111+) puts H-Group-specific filtering (DiscardKnownTrash priority, early-game restriction, endgame
   force-play) in the search layer — AGENTS.md says that belongs behind a ConventionSet hook.

Tier 3 — smaller

5. player_knowledge.rs: combined_possible_identities vs _with_visible are ~95 near-identical lines; effective_inferred_mask rescans all
   hypotheses per card. 6. PlayerPOV default impl reaches past the POV into raw signals fields. 7. Evaluator has 27 flat weights with no semantic
   grouping. 8. Two on-by-default quiescence feature flags with a hardcoded budget (:723) — promote or document. 9. ~30-line
   matches_clue/knowledge_updates boilerplate repeated across save techs. 10. overview.md:15 still calls KnowledgeAwareGameState a "WIP wrapper" —
   it's the settled core.
