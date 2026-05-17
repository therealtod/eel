use std::path::PathBuf;

use eel::engine::convention::hgroup::h_group_convention_set::HGroupConventionSet;
use eel::engine::convention::hgroup::tech::blind_play::BlindPlay;
use eel::engine::convention::hgroup::tech::critical_save::{ColorCriticalSave, RankCriticalSave};
use eel::engine::convention::hgroup::tech::delayed_play_clue::DelayedPlayClue;
use eel::engine::convention::hgroup::tech::direct_play_clue::DirectPlayClue;
use eel::engine::convention::hgroup::tech::discard_chop::DiscardChop;
use eel::engine::convention::hgroup::tech::five_save::FiveSave;
use eel::engine::convention::hgroup::tech::play_known_playable::PlayKnownPlayable;
use eel::engine::convention::hgroup::tech::simple_finesse::SimpleFinesse;
use eel::engine::convention::hgroup::tech::simple_prompt::SimplePrompt;
use eel::engine::convention::hgroup::tech::two_save::TwoSave;
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
fn should_not_start_game_with_bad_touch_for_no_reason() {
    let action = engine_action_at_turn("starts_with_bad_touch.json", 0);
    if let GameAction::Clue {
        player_index: 1,
        clue: Clue {
            clue_type: ClueType::Color,
            clue_value: 0,
        },
        ..
    } = action {
        panic!("expected a clue that does not bad touch, got: {action:?}");
    }
}
