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

fn build_convention_set() -> HGroupConventionSet {
    HGroupConventionSet::new(vec![
        Box::new(PlayKnownPlayable),
        Box::new(BlindPlay),
        Box::new(DirectPlayClue),
        Box::new(DelayedPlayClue),
        Box::new(SimplePrompt),
        Box::new(SimpleFinesse),
        Box::new(ColorCriticalSave),
        Box::new(RankCriticalSave),
        Box::new(FiveSave),
        Box::new(TwoSave),
        Box::new(DiscardChop),
    ])
}

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
    let conv = build_convention_set();
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

/// Scenario (play_clues_trash_1.json):
///
/// 3 players. At turn 2 (Cathy/Bot2's turn) Bot1 already holds a touched, unidentified
/// b1 at deck[8] (from the rank-1 clue at turn 0). After playing deck[6] on turn 1, Bot1
/// drew deck[15] which happens to also be b1. The engine was previously cluing deck[15]
/// as a play clue — a second copy of an already-gotten card. Any action is acceptable
/// here except one that touches deck[15].
#[test]
fn the_engine_should_not_play_clue_second_copy_of_b1() {
    let action = engine_action_at_turn("play_clues_trash_1.json", 2);
    if let GameAction::Clue {
        touched_card_deck_indexes,
        ..
    } = &action
    {
        assert!(
            !touched_card_deck_indexes.contains(&15),
            "engine clued a second copy of b1 (deck[15] already gotten via deck[8]): {action:?}"
        );
    }
}
