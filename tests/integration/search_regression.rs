use crate::common::utils::search_best_action;
use eel::game::action::game_action::GameAction;
use eel::game::clue::Clue;
use eel::game::clue_type::ClueType;

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

#[test]
fn play_known_playable_even_when_foreseeing_stall() {
    let action = search_best_action("play_known_playable_when_foreseeing_stall");
    assert!(
        matches!(
            action,
            GameAction::Play {
                card_deck_index: 3,
                ..
            }
        ),
        "expected play of known B1 (deck 3), got: {action:?}"
    );
}

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
#[ignore]
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

#[test]
fn should_avoid_bad_touch() {
    let action = search_best_action("should_avoid_bad_touch");
    if let GameAction::Clue {
        player_index: 1,
        clue: Clue {
            clue_type: ClueType::Rank,
            clue_value: 1,
        },
        ..
    } = action
    {
        panic!("Alice decided to clue both b1s, generating a bad touch: {action:?}")
    } else {
        println!("Alice choose the following action: {:?}", action);
    }
}

#[test]
fn should_not_give_a_play_clue_that_looks_like_a_save() {
    let action = search_best_action("avoid_giving_play_clues_that_look_like_saves");
    if let GameAction::Clue {
        player_index: 1,
        clue: Clue {
            clue_type: ClueType::Color,
            clue_value: 0,
        },
        ..
    } = action
    {
        panic!(
            "Alice decided to clue red to play clue r3. This will look like a r4 critical save from Bob's POV: {action:?}"
        )
    } else {
        println!("Alice choose the following action: {:?}", action);
    }
}

#[test]
fn should_not_steal_a_play_clue_from_bob_and_rather_save_his_chop() {
    let action = search_best_action("avoid_stealing_clue");
    match action {
        GameAction::Clue {
            player_index: 1,
            clue:
                Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 5,
                },
            ..
        } => {
            println!("Alice choose to save Bob's chop. Action: {:?}", action);
        }
        GameAction::Clue {
            player_index: 2,
            clue:
                Clue {
                    clue_type: ClueType::Color,
                    clue_value: 0,
                },
            ..
        } => {
            panic!(
                "Alice decided to clue Cathy's red 1, Depriving Bob of reasonable actions and putting his chop in danger of being discarded : {action:?}"
            )
        }
        GameAction::Discard {
            player_index: 0,
            card_deck_index: 1,
            ..
        } => {
            panic!(
                "Alice chose to aggressively discard even though it's not as safe as saving Bob's chop {action:?}"
            )
        }
        _ => println!("Alice chose the following action: {:?}", action),
    }
}
#[test]
fn should_choose_the_most_efficient_play_clue() {
    let action = search_best_action("should_choose_the_most_efficient_play_clue");
    match action {
        GameAction::Clue {
            player_index: 1,
            clue:
                Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 2,
                },
            ..
        } => println!(
            "Alice correctly chose to clue rank-2 to Bob, to pick up an extra useful card (r2) {action:?}"
        ),
        GameAction::Clue {
            player_index: 1,
            clue:
                Clue {
                    clue_type: ClueType::Color,
                    clue_value: 1,
                },
            ..
        } => panic!(
            "Alice gave an inferior play clue (grabbing ony one card with the clue), rather than getting an additional good card (r2)   {action:?}"
        ),
        _ => panic!("Alice chose the following action: {action:?}"),
    }
}

#[test]
fn should_prefer_tempo_clue_over_hard_burn() {
    let action = search_best_action("should_prefer_tempo_clue_over_hard_burn");
    match action {
        GameAction::Clue {
            player_index: 1,
            clue:
                Clue {
                    clue_type: ClueType::Color,
                    clue_value: 4,
                },
            ..
        } => println!("Alice has chosen correctly: {:?}", action),
        GameAction::Clue {
            player_index: 1,
            clue:
                Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 5,
                },
            ..
        } => panic!("Alice has decided to just burn a clue {action:?}"),
        _ => panic!("Alice has chosen the following action: {:?}", action),
    }
}

#[test]
fn should_prefer_finesse_if_no_drawback() {
    let action = search_best_action("should_prefer_finesse_if_no_drawback");
    if let GameAction::Clue {
        player_index: 2,
        clue: Clue {
            clue_type: ClueType::Color,
            clue_value: 3,
        },
        ..
    } = action
    {
        println!("Alice choose to give a blue finesse: {:?}", action);
    } else {
        panic!("Alice choose the following action: {:?}", action);
    }
}

#[test]
fn plays_known_playable_despite_critical_dry_downstream() {
    let action = search_best_action("plays_known_playable_despite_critical_dry_downstream");
    assert!(
        matches!(
            action,
            GameAction::Play {
                card_deck_index: 4,
                ..
            }
        ),
        "expected play of known R1 (deck 4) rather than a horizon-driven stall, got: {action:?}"
    );
}

#[test]
fn should_avoid_potential_1s_bad_touch() {
    let action = search_best_action("should_avoid_potential_1s_bad_touch");
    match action {
        GameAction::Play {
            player_index: 0,
            card_deck_index: 0,
            ..
        } => println!("Alice choose to play slot 5 (r1): {:?}", action),
        GameAction::Play {
            player_index: 0,
            card_deck_index: 3,
            ..
        } => println!("Alice choose to play slot 2 (b1): {:?}", action),
        _ => panic!("Alice chose the following action: {:?}", action),
    }
}

#[test]
fn should_prompt_over_hard_burn() {
    let action = search_best_action("should_prompt_over_hard_burn");
    match action {
        GameAction::Clue {
            player_index: 1,
            clue:
                Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 4,
                },
            ..
        } => println!("Alice choose to prompt g3 by cluing g4: {:?}", action),
        _ => panic!("Alice chose the following action: {:?}", action),
    }
}

#[test]
#[ignore]
fn should_avoid_duplicate_ones() {
    let action = search_best_action("should_avoid_duplicate_ones");
    match action {
        GameAction::Clue {
            player_index: 2,
            clue:
                Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 1,
                },
            ..
        } => panic!("Alice has chosen to clue duplicate b1s: {:?}", action),
        _ => println!("Alice has chosen the following action: {:?}", action),
    }
}

#[test]
fn should_play_instead_of_discarding() {
    let action = search_best_action("should_play_instead_of_discarding");
    match action {
        GameAction::Play {
            player_index: 0, ..
        } => println!("Alice has chosen play her known playable p4: {:?}", action),
        _ => panic!("Alice has chosen the following action: {:?}", action),
    }
}

#[test]
fn should_clue_bob_instead_of_skipping_to_cathy() {
    let action = search_best_action("should_clue_bob_instead_of_skipping_to_cathy");
    match action {
        GameAction::Clue {
            player_index: 1,
            clue:
                Clue {
                    clue_type: ClueType::Color,
                    clue_value: 0,
                },
            ..
        } => println!("Alice has chosen to play clue Bob's 2: {:?}", action),
        _ => panic!("Alice has chosen the following action: {:?}", action),
    }
}

#[test]
fn should_not_clue_duplicate_y1() {
    let action = search_best_action("should_not_clue_duplicate_y1");
    match action {
        GameAction::Clue {
            player_index: 2,
            clue:
            Clue {
                clue_type: ClueType::Rank,
                clue_value: 2,
            },
            ..
        } => println!("Alice has chosen to start with a 2-save on Cathy: {:?}", action),
        _ => panic!("Alice has chosen the following action: {:?}", action),
    }
}

#[test]
fn should_pick_up_y1_without_bad_touch() {
    let action = search_best_action("should_pick_up_y1_without_bad_touch");
    match action {
        GameAction::Clue {
            player_index: 1,
            clue:
                Clue {
                    clue_type: ClueType::Color,
                    clue_value: 1,
                },
            ..
        } => println!(
            "Alice has chosen to pick up y1 without bad touch: {:?}",
            action
        ),
        _ => panic!("Alice has chosen the following action: {:?}", action),
    }
}

#[test]
fn should_avoid_the_bad_touch_option() {
    let action = search_best_action("should_avoid_the_bad_touch_option");
    match action {
        GameAction::Clue {
            player_index: 1,
            clue:
            Clue {
                clue_type: ClueType::Color,
                clue_value: o,
            },
            ..
        } => println!(
            "Alice has chosen to pick up r1 from Bob's hand without bad touch: {:?}",
            action
        ),
        _ => panic!("Alice has chosen the following action: {:?}", action),
    }
}