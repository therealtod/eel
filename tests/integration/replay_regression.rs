use std::path::PathBuf;

use crate::common;
use eel::engine::convention::hgroup::h_group_convention_set::HGroupConventionSet;
use eel::engine::knowledge::player_pov::PlayerPOV;
use eel::engine::replay::reconstruct::ReplayRunner;
use eel::engine::tree_action_selection_strategy::TreeActionSelectionStrategy;
use eel::game::action::game_action::GameAction;
use eel::game::clue::Clue;
use eel::game::clue_type::ClueType;
use eel::hanablive::dto::game::Game;

fn path_for(rel: &str) -> PathBuf {
    let name = rel.strip_suffix(".json").unwrap_or(rel);
    [env!("CARGO_MANIFEST_DIR"), "tests", "fixtures", "regression", name, "replay.json"]
        .iter()
        .collect()
}

/// Load a hanab.live replay JSON, step to `turn`, and return the engine's
/// recommended action at that position.
fn engine_action_at_turn(replay_path: &str, turn: usize) -> GameAction {
    common::init_tracing();
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

#[test]
fn should_understand_delayed_play_clue() {
    let action = engine_action_at_turn("should_understand_delayed_play_clue.json", 7);
    if let GameAction::Play {
        player_index: 0,
        card_deck_index: 3,
        ..
    } = action
    {
        panic!(
            "Alice should not play her clued 2 before playing her clued 1. Instead she chose: {action:?}"
        );
    }
}

#[test]
fn delayed_play_clue_admits_full_rank2_union_on_focus() {
    let json = std::fs::read_to_string(path_for("should_understand_delayed_play_clue.json"))
        .expect("read replay");
    let game = Game::from_json(&json).expect("parse replay");
    let conv = HGroupConventionSet::default();
    let mut runner = ReplayRunner::from_hanablive(&game, &conv).expect("build runner");
    runner.step_to_turn(7).expect("step to turn 7");

    let active = runner.game.table_state.active_player_index;
    assert_eq!(active, 0, "Alice (player 0) should be on turn");
    let pov = runner.game.player_pov(active);
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
fn should_discard_less_aggressively() {
    let action = engine_action_at_turn("should_discard_less_aggressively.json", 9);
    if let GameAction::Discard {
        player_index: 2, ..
    } = action
    {
        panic!("Cathy should not choose to discard. Instead got: {action:?}");
    }
}

#[test]
fn should_choose_finesse_over_direct_play_clue() {
    let action = engine_action_at_turn("should_choose_finesse_over_direct_play_clue.json", 13);
    if let GameAction::Clue {
        player_index: 2,
        clue: Clue {
            clue_type: ClueType::Color,
            clue_value: 1,
        },
        ..
    } = action
    {
        println!("Alice chose correctly")
    } else {
        panic!("Alice should give a finesse to get both y1 and y2. Instead got: {action:?}");
    }
}

#[test]
fn should_wait_for_r2() {
    let action = engine_action_at_turn("should_wait_for_r2.json", 29);
    if let GameAction::Play {
        player_index: 1,
        card_deck_index: 27,
        ..
    } = action
    {
        panic!(
            "Bob should entertain that the rank-3 clue is a prompt on Cathy and should not play his slot 1. Instead got: {action:?}"
        )
    }
}

#[test]
fn should_not_steal_clue() {
    let action = engine_action_at_turn("should_not_steal_clue.json", 20);
    match action {
        GameAction::Clue {
            player_index: 2,
            clue:
                Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 5,
                },
            ..
        } => {
            println!("Bob choose to save Cathy's chop. Action: {:?}", action);
        }
        GameAction::Clue {
            player_index: 0,
            clue:
                Clue {
                    clue_type: ClueType::Color,
                    clue_value: 0,
                },
            ..
        } => {
            panic!(
                "Bob decided to clue Alice's red 1, Depriving Cathy of reasonable actions and putting her chop in danger of being discarded : {action:?}"
            )
        }
        GameAction::Discard {
            player_index: 0,
            card_deck_index: 1,
            ..
        } => {
            panic!(
                "Bob chose to aggressively discard even though it's not as safe as saving Cathy's chop {action:?}"
            )
        }
        _ => println!("Bob chose the following action: {:?}", action),
    }
}

#[test]
fn should_play_all_1s() {
    let action = engine_action_at_turn("should_play_all_1s.json", 8);
    match action {
        GameAction::Play {
            player_index: 1,
            card_deck_index: 7,
            ..
        } => {
            println!(
                "Bob played one of his unknown 1s (y1). Action: {:?}",
                action
            );
        }
        GameAction::Play {
            player_index: 1,
            card_deck_index: 9,
            ..
        } => {
            println!(
                "Bob played one of his unknown 1s (g1). Action: {:?}",
                action
            );
        }
        _ => panic!("Bob chose the following action: {:?}", action),
    }
}

#[test]
fn should_not_play_known_trash() {
    let action = engine_action_at_turn("should_not_play_known_trash.json", 44);
    match action {
        GameAction::Play {
            player_index: 1,
            card_deck_index: 8,
            ..
        } => panic!("Bob known trash. Action: {:?}", action),
        _ => println!("Bob chose the following action: {:?}", action),
    }
}

#[test]
fn should_not_discard_off_chop() {
    let action = engine_action_at_turn("should_not_discard_off_chop.json", 19);
    match action {
        GameAction::Discard {
            player_index: 0,
            card_deck_index: 25,
            ..
        } => panic!("Alice discarded off-chop. Action: {:?}", action),
        _ => println!("Alice chose the following action: {:?}", action),
    }
}

#[test]
fn should_not_discard_with_playables_in_hand() {
    let action = engine_action_at_turn("should_not_discard_with_playables_in_hand.json", 13);
    match action {
        GameAction::Play {
            player_index: 0,
            card_deck_index: 3,
            ..
        } => {}
        GameAction::Discard {
            player_index: 0, ..
        } => panic!(
            "Alice discarded with a known-playable card (b1) in her hand. Action: {:?}",
            action
        ),
        _ => panic!(
            "Alice chose neither to play b1 nor discard. Action: {:?}",
            action
        ),
    }
}

#[test]
fn should_prioritize_demonstrating_finesse() {
    let action = engine_action_at_turn("should_prioritize_demonstrating_finesse.json", 7);
    match action {
        GameAction::Play {
            player_index: 0,
            card_deck_index: 3,
            ..
        } => println!(
            "Alice blind-played y1 from her finesse position. Action: {:?}",
            action
        ),
        _ => panic!("Alice chose the following action. Action: {:?}", action),
    }
}

#[test]
fn should_assign_the_correct_identity_to_the_focus() {
    let action = engine_action_at_turn("should_assign_the_correct_identity_to_the_focus.json", 46);
    match action {
        GameAction::Play {
            player_index: 0,
            card_deck_index: 32,
            ..
        } => println!(
            "Alice correctly played the focus of the received green clue: {:?}",
            action
        ),
        _ => panic!("Alice chose the following action. Action: {:?}", action),
    }
}

#[test]
fn should_clue_instead_of_discarding() {
    let action = engine_action_at_turn("should_clue_instead_of_discarding.json", 44);
    match action {
        GameAction::Clue {
            player_index: 2,
            clue:
                Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 2,
                },
            ..
        } => println!("Bob has chosen to play clue b2: {:?}", action),
        _ => panic!("Bob has chosen the following action: {:?}", action),
    }
}

#[test]
fn should_play_instead_of_discarding() {
    let action = engine_action_at_turn("should_play_instead_of_discarding.json", 18);
    match action {
        GameAction::Play {
            player_index: 2,
            card_deck_index: 15,
            ..
        } => println!(
            "Cathy has chosen to play her known-playable p4: {:?}",
            action
        ),
        _ => panic!("Cathy has chosen the following action: {:?}", action),
    }
}

#[test]
#[ignore]
fn should_play_clue_alice() {
    let action = engine_action_at_turn("should_play_clue_alice.json", 3);
    match action {
        GameAction::Clue {
            player_index: 0,
            clue:
                Clue {
                    clue_type: ClueType::Color,
                    clue_value: 0,
                },
            ..
        } => println!("Cathy has chosen to play clue Alice: {:?}", action),
        _ => panic!("Cathy has chosen the following action: {:?}", action),
    }
}

#[test]
#[ignore]
fn should_play_known_playable_1() {
    let action = engine_action_at_turn("should_play_known_playable_1.json", 8);
    match action {
        GameAction::Play {
            player_index: 0,
            card_deck_index: 6,
            ..
        } => println!("Bob has chose to play his known playable 1: {:?}", action),
        _ => panic!("Bob has chosen the following action: {:?}", action),
    }
}

#[test]
fn should_play_her_known_playable_and_let_bob_clue() {
    let action = engine_action_at_turn("should_play_her_known_playable_and_let_bob_clue.json", 13);
    match action {
        GameAction::Play {
            player_index: 0,
            card_deck_index: 2,
            ..
        } => println!("Alice has decided to play her known-playable 3 Cathy: {:?}", action),
        _ => panic!("Alice has chosen the following action: {:?}", action),
    }
}
