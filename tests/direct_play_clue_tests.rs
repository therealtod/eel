mod common;

// Scenario 9: r1 played, only direct play clue is r2 on Cathy
#[test]
fn scenario9_r1_is_played() {
    let (table_state, static_game_data) = common::load_scenario(9);
    let playable = table_state.playable_cards(&static_game_data);
    assert!(playable & (1 << 1) != 0); // R2 is next
}

#[test]
fn scenario9_cathy_has_r2() {
    let (table_state, _) = common::load_scenario(9);
    // Cathy's hand: ["p5", "p3", "y4", "r2", "g3"] — deck indexes 10..14
    // r2 = card id 1, at deck index 13
    let empathy = table_state.deck.get_global_empathy(13);
    assert_eq!(1u64 << 1, empathy); // R2
}

// Scenario 10: r1 played, g1+g2 played, multiple direct play clues available
#[test]
fn scenario10_r2_and_g3_are_playable() {
    let (table_state, static_game_data) = common::load_scenario(10);
    let playable = table_state.playable_cards(&static_game_data);
    assert!(playable & (1 << 1) != 0);  // R2
    assert!(playable & (1 << 12) != 0); // G3
}

#[test]
fn scenario10_has_three_players() {
    let (_, static_game_data) = common::load_scenario(10);
    assert_eq!(3, static_game_data.number_of_players);
}

// Scenario 23: r1 played, Alice can receive a direct play clue (r2)
#[test]
fn scenario23_r1_is_played() {
    let (table_state, static_game_data) = common::load_scenario(23);
    let playable = table_state.playable_cards(&static_game_data);
    assert!(playable & (1 << 1) != 0); // R2 is next
}
