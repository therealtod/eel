mod common;

// Scenario 11: one player (Cathy) has a 5 on chop
#[test]
fn scenario11_has_three_players() {
    let (_, static_game_data) = common::load_scenario(11);
    assert_eq!(3, static_game_data.number_of_players);
}

#[test]
fn scenario11_r1_is_played() {
    let (table_state, static_game_data) = common::load_scenario(11);
    let playable = table_state.playable_cards(&static_game_data);
    assert!(playable & (1 << 1) != 0); // R2 is next
    assert!(playable & 1 == 0);        // R1 is no longer playable
}

#[test]
fn scenario11_cathy_chop_is_b5() {
    let (table_state, _) = common::load_scenario(11);
    // Cathy's hand: ["p5", "p3", "y4", "r2", "b5"] — deck indexes 10..14
    // chop = oldest unclued = deck index 10 (p5) ... actually slot 5 = deck index 14 = b5
    // b5 = card id 19
    let empathy = table_state.deck.get_global_empathy(14);
    assert_eq!(1u64 << 19, empathy); // B5
}

// Scenario 20: multiple players with 5s on chop
#[test]
fn scenario20_has_four_players() {
    let (_, static_game_data) = common::load_scenario(20);
    assert_eq!(4, static_game_data.number_of_players);
}

#[test]
fn scenario20_r1_is_played() {
    let (table_state, static_game_data) = common::load_scenario(20);
    let playable = table_state.playable_cards(&static_game_data);
    assert!(playable & (1 << 1) != 0); // R2 is next
}
