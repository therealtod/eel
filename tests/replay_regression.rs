use std::path::PathBuf;

use eel::engine::convention::hgroup::h_group_convention_set::HGroupConventionSet;
use eel::engine::knowledge::player_pov::PlayerPOV;
use eel::engine::replay::reconstruct::ReplayRunner;
use eel::engine::tree_action_selection_strategy::TreeActionSelectionStrategy;
use eel::external::hanablive::Game;
use eel::game::action::game_action::GameAction;
use eel::game::clue::Clue;
use eel::game::clue_type::ClueType;

fn path_for(rel: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "tests", "replays", rel]
        .iter()
        .collect()
}

/// Load a hanab.live replay JSON, step to `turn`, and return the engine's
/// recommended action at that position.
fn engine_action_at_turn(replay_path: &str, turn: usize) -> GameAction {
    let json = std::fs::read_to_string(path_for(replay_path))
        .unwrap_or_else(|e| panic!("could not read replay {replay_path}: {e}"));
    let game = Game::from_json(&json)
        .unwrap_or_else(|e| panic!("could not parse replay {replay_path}: {e}"));
    let conv = HGroupConventionSet::default();
    let mut runner = ReplayRunner::from_hanablive(&game, &conv)
        .unwrap_or_else(|e| panic!("could not build runner from {replay_path}: {e}"));
    runner
        .step_to_turn(turn)
        .unwrap_or_else(|e| panic!("could not step to turn {turn} in {replay_path}: {e}"));
    let strategy = TreeActionSelectionStrategy::default();
    runner.engine_recommendation(&strategy)
}

// ── Replay regression tests ───────────────────────────────────────────────────
//
// Add tests here as failing games are discovered. Mark with `#[ignore]` while
// the underlying bug is not yet fixed; remove `#[ignore]` once the engine
// passes. See tests/replays/README.md for the workflow.

// Example of how to add a failing-game test once a bug is identified:
//
// #[test]
// #[ignore = "engine slowplays g3 instead of saving b5 on chop"]
// fn slowplay_at_turn_22_should_save_b5() {
//     let action = engine_action_at_turn("game_0042.json", 22);
//     assert!(
//         matches!(
//             action,
//             GameAction::Clue {
//                 clue: eel::game::clue::Clue {
//                     clue_type: eel::game::clue_type::ClueType::Rank,
//                     clue_value: 5,
//                 },
//                 ..
//             }
//         ),
//         "expected rank-5 save clue to Bob, got: {action:?}"
//     );
// }

#[test]
fn should_understand_delayed_play_clue() {
    let action = engine_action_at_turn("should_understand_delayed_play_clue.json", 6);
    if let GameAction::Play {
        player_index: 0,
        card_deck_index: 3,
        ..
    } = action {
        panic!("Alice should not play her clued 2 before playing her clued 1. Instead she chose: {action:?}");
    }
}

/// After the same rank-2 clue, Alice's empathy on the focused slot 2 (deck 3)
/// must span the full {R2, Y2, G2, B2, P2} candidate set: `DirectPlayClue`
/// contributes {B2, P2} (immediately playable), and `DelayedPlayClue` contributes
/// {R2, Y2, G2} via the per-connecting-id sub-hypotheses keyed on her known
/// playable slot 1 (empathy {R1, Y1, G1}).
#[test]
fn delayed_play_clue_admits_full_rank2_union_on_focus() {
    let json = std::fs::read_to_string(path_for("should_understand_delayed_play_clue.json"))
        .expect("read replay");
    let game = Game::from_json(&json).expect("parse replay");
    let conv = HGroupConventionSet::default();
    let mut runner = ReplayRunner::from_hanablive(&game, &conv).expect("build runner");
    runner.step_to_turn(6).expect("step to turn 6");

    let active = runner.game.table_state.active_player_index;
    assert_eq!(active, 0, "Alice (player 0) should be on turn");
    let pov = runner.game.player_pov(active);
    // NO_VARIANT id layout: suit_idx * 5 + (rank - 1).
    // R2=1, Y2=6, G2=11, B2=16, P2=21.
    let expected_rank2_union: u64 = (1 << 1) | (1 << 6) | (1 << 11) | (1 << 16) | (1 << 21);
    let focus_empathy = pov.inferred_identities(3).as_bits();
    assert_eq!(
        focus_empathy & expected_rank2_union,
        expected_rank2_union,
        "Alice's empathy on the focused slot 2 (deck 3) should be the full \
         union of direct ({{B2, P2}}) and delayed ({{R2, Y2, G2}}) candidates; \
         got {focus_empathy:025b}"
    );
}

#[test]
fn should_play_known_playable() {
    let action = engine_action_at_turn("should_play_known_playable.json", 55);
    if let GameAction::Play {
        player_index: 1,
        ..
    } = action {
        println!("Bob correctly chose action: {action:?}");
    } else {
        panic!("Bob should have played a playable card on his slot 5, instead got: {action:?}");
    }
}

#[test]
fn should_not_play_known_trash() {
    let action = engine_action_at_turn("should_not_play_known_trash.json", 33);
    if let GameAction::Play {
        player_index: 0,
        ..
    } = action {
        panic!("Alice has enough empathy on her slot 5 to know it's a trash card (either r3 or r4), So she should not play it. Instead got: {action:?}");
    }
}
