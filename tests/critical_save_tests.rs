mod common;

// Scenario 1: single critical card (p4) on chop
#[test]
fn scenario1_has_p4_in_discard_pile() {
    let (table_state, _) = common::load_scenario(1);
    assert!(table_state.discard_pile.contains_card_with_id(23)); // P4
}

#[test]
fn scenario1_stacks_are_empty() {
    let (table_state, static_game_data) = common::load_scenario(1);
    let playable = table_state.playable_cards(&static_game_data);
    let expected: u64 = 1 | 1 << 5 | 1 << 10 | 1 << 15 | 1 << 20; // all 1s
    assert_eq!(expected, playable);
}

// Scenario 2: no critical cards (empty discard pile)
#[test]
fn scenario2_discard_pile_is_empty() {
    let (table_state, _) = common::load_scenario(2);
    assert_eq!(0, table_state.discard_pile.size());
}

// Scenario 3: 5 on chop, no critical cards
#[test]
fn scenario3_discard_pile_is_empty() {
    let (table_state, _) = common::load_scenario(3);
    assert_eq!(0, table_state.discard_pile.size());
}

// Scenario 4: multiple critical cards on chop (p4, g5, r2, b1, y4 in discard)
#[test]
fn scenario4_has_multiple_critical_cards_in_discard() {
    let (table_state, _) = common::load_scenario(4);
    assert!(table_state.discard_pile.contains_card_with_id(23)); // P4
    assert!(table_state.discard_pile.contains_card_with_id(14)); // G5
    assert!(table_state.discard_pile.contains_card_with_id(1));  // R2
    assert!(table_state.discard_pile.contains_card_with_id(15)); // B1
    assert!(table_state.discard_pile.contains_card_with_id(8));  // Y4
}

#[test]
fn scenario4_has_five_players() {
    let (_, static_game_data) = common::load_scenario(4);
    assert_eq!(5, static_game_data.number_of_players);
}

// Scenario 5: critical 1s (r1 x2, g1 in discard) — critical playables, should not be saved
#[test]
fn scenario5_has_critical_ones_in_discard() {
    let (table_state, _) = common::load_scenario(5);
    assert_eq!(2, table_state.discard_pile.copies_of(0)); // R1 x2
    assert_eq!(1, table_state.discard_pile.copies_of(10)); // G1
}

// Scenario 21: Alice's chop could be a critical card (p4 in discard)
#[test]
fn scenario21_has_p4_in_discard_pile() {
    let (table_state, _) = common::load_scenario(21);
    assert!(table_state.discard_pile.contains_card_with_id(23)); // P4
}
