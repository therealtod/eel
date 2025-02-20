mod common;

// Scenario 6: Cathy knows r1, Bob has a cluable r2 — delayed play clue possible
#[test]
fn scenario6_stacks_are_empty() {
    let (table_state, static_game_data) = common::load_scenario(6);
    let playable = table_state.playable_cards(&static_game_data);
    // All 1s are playable when stacks are empty
    let all_ones: u64 = 1 | 1 << 5 | 1 << 10 | 1 << 15 | 1 << 20;
    assert_eq!(all_ones, playable);
}

#[test]
fn scenario6_cathy_slot3_is_r1() {
    let (table_state, _) = common::load_scenario(6);
    // Cathy's hand: ["b1", "p3", "r1", "y1", "g1"] — deck indexes 10..14
    // r1 = card id 0, at deck index 12
    let empathy = table_state.deck.get_global_empathy(12);
    assert_eq!(1u64, empathy); // R1
}

// Scenario 7: r1 played, Bob has r2 (known) and r4, Donald has r3 (known)
#[test]
fn scenario7_r1_is_played() {
    let (table_state, static_game_data) = common::load_scenario(7);
    let playable = table_state.playable_cards(&static_game_data);
    assert!(playable & (1 << 1) != 0); // R2 is next
}

#[test]
fn scenario7_bob_slot3_is_r2() {
    let (table_state, _) = common::load_scenario(7);
    // Bob's hand: ["r4", "y2", "r2", "b2"] — deck indexes 4..7
    // r2 = card id 1, at deck index 6
    let empathy = table_state.deck.get_global_empathy(6);
    assert_eq!(1u64 << 1, empathy); // R2
}

#[test]
fn scenario7_donald_slot4_is_r3() {
    let (table_state, _) = common::load_scenario(7);
    // Donald's hand: ["y3", "r1", "b4", "r3"] — deck indexes 12..15
    // r3 = card id 2, at deck index 15
    let empathy = table_state.deck.get_global_empathy(15);
    assert_eq!(1u64 << 2, empathy); // R3
}

// Scenario 8: r1 played, r2 known but r3 not fully known — cannot give delayed play clue for r4
#[test]
fn scenario8_r1_is_played() {
    let (table_state, static_game_data) = common::load_scenario(8);
    let playable = table_state.playable_cards(&static_game_data);
    assert!(playable & (1 << 1) != 0); // R2 is next
}

#[test]
fn scenario8_bob_slot3_is_r2() {
    let (table_state, _) = common::load_scenario(8);
    // Bob's hand: ["r4", "y2", "r2", "b2"] — deck indexes 4..7
    let empathy = table_state.deck.get_global_empathy(6);
    assert_eq!(1u64 << 1, empathy); // R2
}

// Scenario 22: Alice can receive a delayed play clue (r2 in her hand, r1 known to Donald)
#[test]
fn scenario22_r1_is_played() {
    let (table_state, static_game_data) = common::load_scenario(22);
    let playable = table_state.playable_cards(&static_game_data);
    assert!(playable & (1 << 1) != 0); // R2 is next
}

#[test]
fn scenario22_alice_slot4_is_r2() {
    let (table_state, _) = common::load_scenario(22);
    // Alice's hand: ["x", "x", "x", "r2"] — deck indexes 0..3
    // r2 = card id 1, at deck index 3
    let empathy = table_state.deck.get_global_empathy(3);
    assert_eq!(1u64 << 1, empathy); // R2
}
