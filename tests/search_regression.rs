mod common;

use eel::engine::action_selection_strategy::ActionSelectionStrategy;
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
use eel::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
use eel::engine::tree_action_selection_strategy::TreeActionSelectionStrategy;
use eel::game::action::game_action::GameAction;
use eel::game::clue::Clue;
use eel::game::clue_type::ClueType;

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

/// Load a named search regression scenario and ask the engine for its best action.
fn search_best_action(name: &str) -> GameAction {
    let (table_state, static_data, team_knowledge, _, _) =
        common::load_scenario_by_name_with_knowledge(&format!("search/{name}"));
    let active = table_state.active_player_index as usize;
    let knowledge = team_knowledge.player(active).clone();
    let pov = LightweightPlayerPOV::new(
        active,
        &knowledge,
        &team_knowledge,
        &table_state,
        &static_data,
    );
    let strategy = TreeActionSelectionStrategy::default();
    let conventions = build_convention_set();
    strategy.select_active_player_action(&pov, &conventions)
}

// 3p, empty stacks. Player 0 knows their slot-1 card is R1 (directly playable).
// No useful clue targets exist. Expected: play deck 4 (R1).
#[test]
fn play_known_playable() {
    let action = search_best_action("play_known_playable");
    assert!(
        matches!(
            action,
            GameAction::Play {
                card_deck_index: 4,
                ..
            }
        ),
        "expected play of known R1 (deck 4), got: {action:?}"
    );
}

// 3p, empty stacks. Player 1's slot-1 card is R1 (visible to player 0).
// Player 0 has no known playable cards. Expected: direct-play clue to player 1 touching R1 (deck 9).
#[test]
fn direct_play_clue_is_top_choice() {
    let action = search_best_action("direct_play_clue_is_top_choice");
    match action {
        GameAction::Clue {
            player_index: 1,
            ref touched_card_deck_indexes,
            ..
        } => {
            assert!(
                touched_card_deck_indexes.contains(&9),
                "expected clue to Bob touching R1 (deck 9), got touches: {touched_card_deck_indexes:?}"
            );
        }
        _ => panic!("expected a direct-play clue to Bob, got: {action:?}"),
    }
}

#[test]
#[ignore]
fn prefers_blue_clue_to_cathy_over_rank1_clue_to_bob() {
    let action = search_best_action("long_term_setup");
    match action {
        GameAction::Clue {
            player_index: 2,
            clue:
                Clue {
                    clue_type: ClueType::Color,
                    clue_value: 3,
                },
            ..
        } => {}
        _ => panic!("expected a blue clue to Cathy, got: {action:?}"),
    }
}

#[test]
fn understands_that_the_efficient_clue_loses_max_score() {
    let action = search_best_action("avoid_killing_critical");
    match action {
        GameAction::Clue {
            player_index: 1,
            clue:
                Clue {
                    clue_type: ClueType::Color,
                    clue_value: 2,
                },
            ..
        } => {}
        GameAction::Clue {
            player_index: 1,
            clue:
                Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 3,
                },
            ..
        } => {}
        GameAction::Discard {
            player_index: 0, ..
        } => {}
        _ => panic!("expected a Critical save to Bob, got: {action:?}"),
    }
}

#[test]
fn does_not_steal_a_finesse_that_bob_could_give() {
    let action = search_best_action("avoid_stealing_finesse");
    match action {
        GameAction::Play {
            player_index: 0,
            card_deck_index: 2,
            ..
        } => {}
        _ => panic!("expected to play a known playable g2 in slot 2, got: {action:?}"),
    }
}

#[test]
fn defers_playing_a_known_playable_to_save_a_critical_card() {
    let action = search_best_action("defer_play_to_save_critical");
    match action {
        GameAction::Clue {
            player_index: 1,
            clue:
                Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 3,
                },
            ..
        } => {}
        _ => panic!("expected a CriticalSave to Bob, got: {action:?}"),
    }
}

#[test]
fn does_not_defer_play_to_save_cathy_when_bob_could_save() {
    let action = search_best_action("avoid_deferring_play_to_save");
    match action {
        GameAction::Play {
            player_index: 0,
            card_deck_index: 3,
            ..
        } => {}
        _ => panic!("expected to play a known playable g2 in slot 2, got: {action:?}"),
    }
}

#[test]
#[ignore]
fn prefers_more_efficient_finesse_over_direct_play_clue() {
    let action = search_best_action("prefer_finesse_over_direct_play_clue_when_more_efficient");
    match action {
        GameAction::Clue {
            player_index: 2,
            clue:
                Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 3,
                },
            ..
        } => {}
        _ => panic!("expected to clue r3 in Cathy's hand as a finesse, got: {action:?}"),
    }
}

#[test]
#[ignore]
fn prefers_to_clue_rank_1_rather_than_picking_up_1s_by_color() {
    let action = search_best_action(
        "does_not_slow_down_the_game_due_to_foreseeing_too_many_discards_cause_of_search_horizon",
    );
    match action {
        GameAction::Clue {
            player_index: 2,
            clue:
                Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 1,
                },
            ..
        } => {}
        _ => panic!("expected to clue rank-1 to Cathy's, got: {action:?}"),
    }
}
