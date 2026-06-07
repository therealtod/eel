use std::path::PathBuf;

use crate::common;
use eel::engine::convention::hgroup::h_group_convention_set::{HGroupConventionSet, HGroupLevel};
use eel::engine::knowledge::player_pov::PlayerPOV;
use eel::engine::replay::reconstruct::ReplayRunner;
use eel::engine::tree_action_selection_strategy::TreeActionSelectionStrategy;
use eel::game::action::game_action::GameAction;
use eel::game::clue::Clue;
use eel::game::clue_type::ClueType;
use eel::game::variant::test_variants::NO_VARIANT;
use eel::hanablive::dto::payload::game::Game;
use eel::hanablive::replay_transcript::{
    engine_action_at_turn as transcript_action_at_turn, synthesize_transcript,
};

fn path_for(rel: &str) -> PathBuf {
    let name = rel.strip_suffix(".json").unwrap_or(rel);
    [
        env!("CARGO_MANIFEST_DIR"),
        "tests",
        "fixtures",
        "regression",
        name,
        "replay.json",
    ]
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
    let conv = HGroupConventionSet::for_level(HGroupLevel::One, &NO_VARIANT);
    let mut runner = ReplayRunner::from_hanablive(&game, &conv)
        .unwrap_or_else(|e| panic!("could not build runner from {replay_path}: {e}"));
    runner
        .step_to_turn(turn)
        .unwrap_or_else(|e| panic!("could not step to turn {turn} in {replay_path}: {e}"));
    let strategy = TreeActionSelectionStrategy::default();
    runner.engine_recommendation(&strategy)
}

/// Like [`engine_action_at_turn`], but reconstructs the game with the *acting* seat's own hand
/// hidden — reproducing the live bot's perspective (it never learns its own card identities).
///
/// The full-deck reconstruction reveals every card, including the acting seat's hand, to all
/// players' empathy; live play does not. A test built this way therefore exercises the same
/// hidden-own-hand state the live bot actually decides from, catching divergences the omniscient
/// reconstruction masks. The hidden seat is discovered by a first pass that finds which player is
/// active at `turn`.
fn engine_action_at_turn_hidden_seat(replay_path: &str, turn: usize) -> GameAction {
    common::init_tracing();
    let json = std::fs::read_to_string(path_for(replay_path))
        .unwrap_or_else(|e| panic!("could not read replay {replay_path}: {e}"));
    let game = Game::from_json(&json)
        .unwrap_or_else(|e| panic!("could not parse replay {replay_path}: {e}"));
    let conv = HGroupConventionSet::for_level(HGroupLevel::One, &NO_VARIANT);

    // First pass: discover which seat acts at `turn`.
    let acting_seat = {
        let mut probe = ReplayRunner::from_hanablive(&game, &conv)
            .unwrap_or_else(|e| panic!("could not build probe runner from {replay_path}: {e}"));
        probe.step_to_turn(turn).unwrap_or_else(|e| {
            panic!("could not step probe to turn {turn} in {replay_path}: {e}")
        });
        probe.game.table_state.active_player_index
    };

    // Second pass: reconstruct with that seat's own hand hidden, as the live bot saw it.
    let mut runner = ReplayRunner::from_hanablive_hidden_seat(&game, &conv, acting_seat)
        .unwrap_or_else(|e| panic!("could not build hidden-seat runner from {replay_path}: {e}"));
    runner
        .step_to_turn(turn)
        .unwrap_or_else(|e| panic!("could not step to turn {turn} in {replay_path}: {e}"));
    let strategy = TreeActionSelectionStrategy::default();
    runner.engine_recommendation(&strategy)
}

/// A/B harness: assert the live `BotState` code path (driven by a transcript synthesized from the
/// same `Game`, with the acting seat's own hand hidden) agrees with the hidden-seat `ReplayRunner`
/// at `turn`.
///
/// Both reconstructions hide the acting seat and reveal teammates, and they recompute identical
/// clue targets, so they operate on the same information — this isolates any discrepancy in the
/// two *implementations* (frame ordering, replacement-draw handling, turn resync). A real
/// divergence that depends on server-only data (e.g. clue `list` order, which the export lacks)
/// is out of this harness's reach by construction; capture a live transcript to chase those.
fn assert_transcript_matches_replay_runner(replay_path: &str, turn: usize) {
    common::init_tracing();
    let json = std::fs::read_to_string(path_for(replay_path))
        .unwrap_or_else(|e| panic!("could not read replay {replay_path}: {e}"));
    let game = Game::from_json(&json)
        .unwrap_or_else(|e| panic!("could not parse replay {replay_path}: {e}"));
    let conv = HGroupConventionSet::for_level(HGroupLevel::One, &NO_VARIANT);

    // Which seat acts at `turn`? That seat is hidden in both reconstructions.
    let acting_seat = {
        let mut probe = ReplayRunner::from_hanablive(&game, &conv)
            .unwrap_or_else(|e| panic!("could not build probe runner from {replay_path}: {e}"));
        probe
            .step_to_turn(turn)
            .unwrap_or_else(|e| panic!("could not step probe to turn {turn}: {e}"));
        probe.game.table_state.active_player_index
    };

    let replay_action = engine_action_at_turn_hidden_seat(replay_path, turn);

    let transcript = synthesize_transcript(&game, acting_seat);
    let transcript_action = transcript_action_at_turn(&transcript, HGroupLevel::One, turn)
        .unwrap_or_else(|e| panic!("transcript replay could not reach turn {turn}: {e}"));

    // Compare via Debug so this works without a `PartialEq` bound; the recommended action's
    // touched-card list is engine-generated identically in both paths, so it matches too.
    assert_eq!(
        format!("{replay_action:?}"),
        format!("{transcript_action:?}"),
        "ReplayRunner (hidden seat {acting_seat}) and synthesized-transcript BotState must agree \
         at turn {turn} of {replay_path}"
    );
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
    let conv = HGroupConventionSet::for_level(HGroupLevel::One, &NO_VARIANT);
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
        println!("Alice chose correctly");
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
            println!("Bob choose to save Cathy's chop. Action: {action:?}");
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
        _ => println!("Bob chose the following action: {action:?}"),
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
            println!("Bob played one of his unknown 1s (y1). Action: {action:?}");
        }
        GameAction::Play {
            player_index: 1,
            card_deck_index: 9,
            ..
        } => {
            println!("Bob played one of his unknown 1s (g1). Action: {action:?}");
        }
        _ => panic!("Bob chose the following action: {action:?}"),
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
        } => panic!("Bob known trash. Action: {action:?}"),
        _ => println!("Bob chose the following action: {action:?}"),
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
        } => panic!("Alice discarded off-chop. Action: {action:?}"),
        _ => println!("Alice chose the following action: {action:?}"),
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
            "Alice discarded with a known-playable card (b1) in her hand. Action: {action:?}"
        ),
        _ => panic!("Alice chose neither to play b1 nor discard. Action: {action:?}"),
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
        } => println!("Alice blind-played y1 from her finesse position. Action: {action:?}"),
        _ => panic!("Alice chose the following action. Action: {action:?}"),
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
        } => println!("Alice correctly played the focus of the received green clue: {action:?}"),
        _ => panic!("Alice chose the following action. Action: {action:?}"),
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
        } => println!("Bob has chosen to play clue b2: {action:?}"),
        _ => panic!("Bob has chosen the following action: {action:?}"),
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
        } => println!("Cathy has chosen to play her known-playable p4: {action:?}"),
        _ => panic!("Cathy has chosen the following action: {action:?}"),
    }
}

#[test]
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
        } => println!("Cathy has chosen to play clue Alice's r2: {action:?}"),
        GameAction::Clue {
            player_index: 0,
            clue:
                Clue {
                    clue_type: ClueType::Color,
                    clue_value: 1,
                },
            ..
        } => println!("Cathy has chosen to play clue Alice's y1: {action:?}"),
        _ => panic!("Cathy has chosen the following action: {action:?}"),
    }
}

#[test]
fn should_not_discard() {
    let action = engine_action_at_turn("should_not_discard.json", 8);
    match action {
        GameAction::Play {
            player_index: 0,
            card_deck_index: 6,
            ..
        } => println!("Bob has chosen to play his known playable 1: {action:?}"),
        GameAction::Clue {
            player_index: 0,
            clue:
                Clue {
                    clue_type: ClueType::Color,
                    clue_value: 0,
                },
            ..
        } => println!("Bob has chosen to clue red to Alice: {action:?}"),
        _ => panic!("Bob has chosen the following action: {action:?}"),
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
        } => println!("Alice has decided to play her known-playable 3 Cathy: {action:?}"),
        _ => panic!("Alice has chosen the following action: {action:?}"),
    }
}

#[test]
fn should_pick_up_y1_without_bad_touch() {
    let action = engine_action_at_turn("should_pick_up_y1_without_bad_touch.json", 2);
    // Active player is Bob (1); the lone y1 sits in Cathy's (player 2) hand alongside three
    // P1 duplicates. The clean pickup is a Color/1 clue to Cathy touching only the y1 — a
    // rank-1 clue would bad-touch the duplicate P1s.
    match action {
        GameAction::Clue {
            player_index: 2,
            clue:
                Clue {
                    clue_type: ClueType::Color,
                    clue_value: 1,
                },
            ..
        } => println!("Bob has chosen to pick up y1 without bad touch: {action:?}"),
        _ => panic!("Bob has chosen the following action: {action:?}"),
    }
}

#[test]
fn should_not_clue_1s_to_cathy() {
    let action = engine_action_at_turn("should_not_clue_1s_to_cathy.json", 17);
    match action {
        GameAction::Discard {
            player_index: 1, ..
        } => println!("Bob has chosen to discard his chop: {action:?}"),
        _ => panic!("Bob has chosen the following action: {action:?}"),
    }
}

#[test]
fn should_wait_for_g1_duplicate_to_discard() {
    let action = engine_action_at_turn("should_wait_for_g1_duplicate_to_discard.json", 10);
    match action {
        GameAction::Clue {
            player_index: 2,
            clue:
                Clue {
                    clue_type: ClueType::Color,
                    clue_value: 0,
                },
            ..
        } => println!(
            "Alice has chosen to clue Cathy's red 2 and calling for Bob's chop to discard: {action:?}"
        ),
        _ => panic!("Alice has chosen the following action: {action:?}"),
    }
}

#[test]
fn should_discard_known_trash_before_chop() {
    let action = engine_action_at_turn("should_discard_known_trash_before_chop.json", 34);
    match action {
        GameAction::Discard {
            player_index: 0,
            card_deck_index: 4,
            ..
        } => println!("Alice has chosen to discard her known trash: {action:?}"),
        GameAction::Discard {
            player_index: 0,
            card_deck_index: 23,
            ..
        } => panic!(
            "Alice has chosen to discard her chop over safely discarding known trash: {action:?}"
        ),
        _ => panic!("Alice has chosen the following action: {action:?}"),
    }
}

#[test]
fn should_not_give_ooo_prompt() {
    let action = engine_action_at_turn("should_not_give_ooo_prompt.json", 44);
    match action {
        GameAction::Clue {
            player_index: 0,
            clue:
                Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 3,
                },
            ..
        } => println!("Bob has chosen to play clue a playable r3: {action:?}"),
        _ => panic!("Bob has chosen the following action: {action:?}"),
    }
}

/// Same scenario as [`should_not_give_ooo_prompt`], but reconstructed with the acting seat's own
/// hand hidden — i.e. the perspective the live bot actually plays from. The omniscient full-deck
/// reconstruction lets the engine's teammate model see the bot's own hand, which live play cannot;
/// this test exercises the realistic hidden-own-hand state to expose divergences between live and
/// the full-deck replay.
#[test]
fn should_not_give_ooo_prompt_hidden_seat() {
    let action = engine_action_at_turn_hidden_seat("should_not_give_ooo_prompt.json", 44);
    match action {
        GameAction::Clue {
            player_index: 0,
            clue:
                Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 3,
                },
            ..
        } => println!("Bob has chosen to play clue a playable r3: {action:?}"),
        _ => panic!("Bob has chosen the following action: {action:?}"),
    }
}

/// Equivalence guard across every regression fixture: the live `BotState` code path (synthesized
/// transcript, acting seat hidden) must agree with the hidden-seat `ReplayRunner` at each fixture's
/// decision turn. This keeps the two reconstructions from silently drifting apart — a divergence
/// here means a real bug in one path's frame/draw/turn handling, the class that makes live behave
/// differently from selfplay.
#[test]
fn transcript_matches_replay_runner_across_fixtures() {
    let cases = [
        ("should_understand_delayed_play_clue.json", 7),
        ("should_discard_less_aggressively.json", 9),
        ("should_choose_finesse_over_direct_play_clue.json", 13),
        ("should_wait_for_r2.json", 29),
        ("should_not_steal_clue.json", 20),
        ("should_play_all_1s.json", 8),
        ("should_not_play_known_trash.json", 44),
        ("should_not_discard_off_chop.json", 19),
        ("should_not_discard_with_playables_in_hand.json", 13),
        ("should_prioritize_demonstrating_finesse.json", 7),
        ("should_assign_the_correct_identity_to_the_focus.json", 46),
        ("should_clue_instead_of_discarding.json", 44),
        ("should_play_instead_of_discarding.json", 18),
        ("should_play_clue_alice.json", 3),
        ("should_not_discard.json", 8),
        ("should_play_her_known_playable_and_let_bob_clue.json", 13),
        ("should_pick_up_y1_without_bad_touch.json", 2),
        ("should_not_clue_1s_to_cathy.json", 17),
        ("should_wait_for_g1_duplicate_to_discard.json", 10),
        ("should_discard_known_trash_before_chop.json", 34),
        ("should_not_give_ooo_prompt.json", 44),
    ];
    for (path, turn) in cases {
        assert_transcript_matches_replay_runner(path, turn);
    }
}

#[test]
fn should_trust_play_clue() {
    let action = engine_action_at_turn_hidden_seat("should_trust_play_clue.json", 6);
    match action {
        GameAction::Play {
            player_index: 1,
            card_deck_index: 7,
            ..
        } => println!("Bob has chosen to play his playable g1: {action:?}"),
        _ => panic!("Bob has chosen the following action: {action:?}"),
    }
}
