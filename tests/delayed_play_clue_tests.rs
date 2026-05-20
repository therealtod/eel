mod common;

use eel::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
use eel::engine::knowledge::player_pov::PlayerPOV;

/// After the same rank-2 clue, Alice's empathy on the focused slot 2 (deck 3)
/// must span the full {R2, Y2, G2, B2, P2} candidate set: `DirectPlayClue`
/// contributes {B2, P2} (immediately playable), and `DelayedPlayClue` contributes
/// {R2, Y2, G2} via the per-connecting-id sub-hypotheses keyed on her known
/// playable slot 1 (empathy {R1, Y1, G1}).
#[test]
#[ignore]
fn delayed_play_clue_admits_full_rank2_union_on_focus() {
    let (table_state, static_data, team_knowledge, history, actions) =
        common::load_scenario_with_knowledge("delayed_play_clue", 1);

    let active = table_state.active_player_index;
    assert_eq!(active, 1, "Bob (player 1) should be on turn");
    let mut bob_table_state = table_state.clone();
    bob_table_state.active_player_index = 1;
    let bob_knowledge = team_knowledge.player(1).clone();
    let bob_pov = LightweightPlayerPOV::new(
        1,
        &bob_knowledge,
        &team_knowledge,
        &bob_table_state,
        &static_data,
    );
    // NO_VARIANT id layout: suit_idx * 5 + (rank - 1).
    // R2=1, Y2=6, G2=11, B2=16, P2=21.
    let expected_rank2_union: u64 = (1 << 1) | (1 << 6) | (1 << 11) | (1 << 16) | (1 << 21);
    let focus_empathy = bob_pov.inferred_identities(3).as_bits();
    assert_eq!(
        focus_empathy & expected_rank2_union,
        expected_rank2_union,
        "Alice's empathy on the focused slot 2 (deck 3) should be the full \
         union of direct ({{B2, P2}}) and delayed ({{R2, Y2, G2}}) candidates; \
         got {focus_empathy:025b}"
    );
}