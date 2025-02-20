mod common;

/// Scenario 1: a board with a single critical card (p4) on chop.
/// 4 players, no-variant, empty stacks, p4 in the trash pile.
#[test]
fn scenario1_loads_correct_number_of_players() {
    let (_, static_game_data) = common::load_scenario(1);
    assert_eq!(4, static_game_data.number_of_players);
}

#[test]
fn scenario1_has_p4_in_discard_pile() {
    let (table_state, _) = common::load_scenario(1);
    // P4 = suit index 4 (purple), rank 4 → card id = 4*5 + 3 = 23
    let p4_id = 23;
    assert!(table_state.discard_pile.contains_card_with_id(p4_id));
}

#[test]
fn scenario1_has_correct_clue_tokens() {
    let (table_state, _) = common::load_scenario(1);
    assert_eq!(5, table_state.clue_token_bank.whole_clue_tokens_count());
}

#[test]
fn scenario1_playing_stacks_are_empty() {
    let (table_state, static_game_data) = common::load_scenario(1);
    // All stacks empty → next playable cards are the 1s of each suit
    let playable = table_state.playable_cards(&static_game_data);
    // R1=0, Y1=5, G1=10, B1=15, P1=20
    let expected: u64 = 1 | 1 << 5 | 1 << 10 | 1 << 15 | 1 << 20;
    assert_eq!(expected, playable);
}
