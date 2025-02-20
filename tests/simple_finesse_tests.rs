mod common;

// Scenario 16: r1 played, b1+b2 played — Bob has r2 on finesse position, Cathy has b4
#[test]
fn scenario16_r2_and_b3_are_playable() {
    let (table_state, static_game_data) = common::load_scenario(16);
    let playable = table_state.playable_cards(&static_game_data);
    assert!(playable & (1 << 1) != 0);  // R2
    assert!(playable & (1 << 17) != 0); // B3
}

#[test]
fn scenario16_bob_newest_card_is_r2() {
    let (table_state, _) = common::load_scenario(16);
    // Bob's hand: ["r2", "b3", "y4", "b1", "r3"] — deck indexes 5..9
    // r2 = card id 1, at deck index 5 (oldest in hand)
    let empathy = table_state.deck.get_global_empathy(5);
    assert_eq!(1u64 << 1, empathy); // R2
}

// Scenario 17: reverse finesse — should not be given
#[test]
fn scenario17_cathy_chop_is_y2() {
    let (table_state, _) = common::load_scenario(17);
    // Cathy's hand: ["r2", "r3", "p2", "p2", "y2"] — deck indexes 10..14
    // y2 = card id 6, at deck index 14
    let empathy = table_state.deck.get_global_empathy(14);
    assert_eq!(1u64 << 6, empathy); // Y2
}

// Scenario 24: teammate (Donald) can give a simple finesse
#[test]
fn scenario24_has_four_players() {
    let (_, static_game_data) = common::load_scenario(24);
    assert_eq!(4, static_game_data.number_of_players);
}

#[test]
fn scenario24_bob_oldest_card_is_r2() {
    let (table_state, _) = common::load_scenario(24);
    // Bob's hand: ["r2", "b3", "y4", "b1"] — deck indexes 4..7
    // r2 = card id 1, at deck index 4 (oldest)
    let empathy = table_state.deck.get_global_empathy(4);
    assert_eq!(1u64 << 1, empathy); // R2
}
