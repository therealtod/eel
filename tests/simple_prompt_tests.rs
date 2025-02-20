mod common;

// Scenario 12: r1 played, b2 played — Bob has r2+p2 clued, Cathy has b3 clued
#[test]
fn scenario12_r2_and_b3_are_playable() {
    let (table_state, static_game_data) = common::load_scenario(12);
    let playable = table_state.playable_cards(&static_game_data);
    assert!(playable & (1 << 1) != 0);  // R2
    assert!(playable & (1 << 17) != 0); // B3
}

#[test]
fn scenario12_bob_slot3_is_r2() {
    let (table_state, _) = common::load_scenario(12);
    // Bob's hand: ["b4", "r4", "r2", "p2", "r1"] — deck indexes 5..9
    // r2 = card id 1, at deck index 7
    let empathy = table_state.deck.get_global_empathy(7);
    assert_eq!(1u64 << 1, empathy); // R2
}

// Scenario 13: wrong prompt that cannot be patched — only b4 prompt is valid
#[test]
fn scenario13_bob_slot3_is_p2_not_r2() {
    let (table_state, _) = common::load_scenario(13);
    // Bob's hand: ["b4", "r4", "p2", "r2", "r1"] — deck indexes 5..9
    // p2 = card id 21, at deck index 7
    let empathy = table_state.deck.get_global_empathy(7);
    assert_eq!(1u64 << 21, empathy); // P2
}

// Scenario 14: wrong prompt that can be patched — both prompts valid
#[test]
fn scenario14_cathy_slot3_is_r2() {
    let (table_state, _) = common::load_scenario(14);
    // Cathy's hand: ["b4", "r4", "r2", "p2", "r1"] — deck indexes 10..14
    // r2 = card id 1, at deck index 12
    let empathy = table_state.deck.get_global_empathy(12);
    assert_eq!(1u64 << 1, empathy); // R2
}

// Scenario 15: wrong prompt that cannot be patched in time
#[test]
fn scenario15_has_four_players() {
    let (_, static_game_data) = common::load_scenario(15);
    assert_eq!(4, static_game_data.number_of_players);
}

#[test]
fn scenario15_cathy_slot3_is_r2() {
    let (table_state, _) = common::load_scenario(15);
    // Cathy's hand: ["r4", "p2", "r2", "b4"] — deck indexes 8..11
    // r2 = card id 1, at deck index 10
    let empathy = table_state.deck.get_global_empathy(10);
    assert_eq!(1u64 << 1, empathy); // R2
}

// Scenario 26: teammate (Donald) can give a simple prompt to Bob
#[test]
fn scenario26_bob_slot3_is_r2() {
    let (table_state, _) = common::load_scenario(26);
    // Bob's hand: ["b4", "r4", "r2", "p2"] — deck indexes 4..7
    // r2 = card id 1, at deck index 6
    let empathy = table_state.deck.get_global_empathy(6);
    assert_eq!(1u64 << 1, empathy); // R2
}
