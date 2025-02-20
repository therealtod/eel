mod common;

// Scenario 17: y2 on Cathy's chop, no other copy visible — should be saved
#[test]
fn scenario17_has_empty_discard_and_b2_played() {
    let (table_state, static_game_data) = common::load_scenario(17);
    assert_eq!(0, table_state.discard_pile.size());
    // r1 and b2 are on the stacks → r2 and b3 are next playable
    let playable = table_state.playable_cards(&static_game_data);
    assert!(playable & (1 << 1) != 0);  // R2 playable
    assert!(playable & (1 << 17) != 0); // B3 playable
}

// Scenario 18: y2 on Cathy's chop but Bob also has y2 (visible rule — cannot save)
#[test]
fn scenario18_has_three_players() {
    let (_, static_game_data) = common::load_scenario(18);
    assert_eq!(3, static_game_data.number_of_players);
}

#[test]
fn scenario18_has_empty_discard() {
    let (table_state, _) = common::load_scenario(18);
    assert_eq!(0, table_state.discard_pile.size());
}

// Scenario 19: both copies of y2 on chop (Bob and Cathy) — both should be saveable
#[test]
fn scenario19_has_three_players() {
    let (_, static_game_data) = common::load_scenario(19);
    assert_eq!(3, static_game_data.number_of_players);
}

// Scenario 27: y2 cannot be saved — Donald (clue giver) can see the other copy in Bob's hand
#[test]
fn scenario27_has_four_players() {
    let (_, static_game_data) = common::load_scenario(27);
    assert_eq!(4, static_game_data.number_of_players);
}

// Scenario 28: y2 on Cathy's chop but Alice already knows she has y2
// Alice's hand: ["x", "y2", "x", "x", "x"] — deck index 1 is y2
#[test]
fn scenario28_alice_slot2_is_revealed_as_y2() {
    let (table_state, _) = common::load_scenario(28);
    // deck index 1 (Alice's second card) should be revealed as y2 (card id 6)
    let empathy = table_state.deck.get_global_empathy(1);
    assert_eq!(1u64 << 6, empathy);
}
