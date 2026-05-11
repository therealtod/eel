# Current Task

Implement `all_players_understand_simple_prompt_semantics()` in `tests/simple_prompt_tests.rs` for scenario 3.

## Scenario 3 key facts
- 3p, stacks=[r1r2r3,_,_,b1b2,p1p2], Alice on turn
- Alice deck4=p3 (purple clue, inferred p3)
- Bob deck9=y4(+4), deck8=b3(+3), deck7=r3(+3), deck6=p3(+3), deck5=r2
- Cathy deck14=b4, deck13=p4, ..., deck10=g3
- Prior action: rank-4 clue Alice→Cathy, touches [14,13], focus=deck14 (b4, away=1)
- Connecting card: b3=deck8; deck9 (y4, rank-4 empathy) is skipped

## Changes made
1. **`delayed_play_clue.rs`**: removed `filter(|&p| p != active)` in `connecting_cards_are_known` → Alice's p3 (deck4, known) now qualifies p4 as delayed play target for Cathy
2. **`simple_prompt.rs`**: added skip-by-empathy check to `is_valid_prompt_situation`; refactored Case 1 of `clue_knowledge_updates` to use `is_valid_prompt_situation` + find first empathy-compatible card (deck8, not deck9)
3. **`tests/common/mod.rs`**: added `variant` param to `team_knowledge_from_scenario`; added clue-narrowing loop applying positive/negative clue marks to holder's `inferred_identities` via `narrow_inferred`

## Still TODO
- Write the test body in `tests/simple_prompt_tests.rs`
  - Part 1: Alice generates rank-4 prompt clue
  - Part 2: Bob gets NarrowPossibilities{deck8, b3_mask}
  - Part 3: Cathy combines SimplePrompt(b4) + DelayedPlayClue(p4) + stand-in(r4); effective mask = r4|b4|p4
- Run tests to verify
