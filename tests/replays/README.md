# Replay regression corpus

## Workflow

1. Copy a failing game from `logs/<score>/game_<N>.json` into this directory with a
   descriptive name (e.g. `slowplay_g3_instead_of_save.json`).
2. Open the replay in hanab.live ("Watch Specific Replay") and find the exact turn
   where the engine makes the wrong decision.
3. Add a `#[test]` in `tests/replay_regression.rs`:

```rust
#[test]
#[ignore] // remove once the bug is fixed
fn slowplay_at_turn_22_should_save_b5() {
    let action = engine_action_at_turn("slowplay_g3_instead_of_save.json", 22);
    assert!(
        matches!(
            action,
            GameAction::Clue { clue: Clue { clue_type: ClueType::Rank, clue_value: 5 }, .. }
        ),
        "expected rank-5 save, got: {action:?}"
    );
}
```

4. Mark the test `#[ignore]` while the bug is not yet fixed.
   Remove `#[ignore]` once the engine produces the correct action.

## Size limit

Keep this corpus small (< 5 games). Only move a replay here when it documents a
tracked bug. Developer-local runs live in `logs/` (git-ignored) and are never
checked in.
