mod common;

use eel::engine::convention::convention_tech::ConventionTech;
use eel::engine::convention::hgroup::tech::delayed_play_clue::DelayedPlayClue;
use eel::engine::convention::hgroup::tech::direct_play_clue::DirectPlayClue;
use eel::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
use eel::engine::knowledge::player_pov::PlayerPOV;

/// After receiving a rank-2 clue, Alice's empathy on the focused slot (deck 3)
/// must span the full {R2, Y2, G2, B2, P2} candidate set: `DirectPlayClue`
/// contributes {B2, P2} (immediately playable), and `DelayedPlayClue` contributes
/// {R2, Y2, G2} via the per-connecting-id sub-hypotheses keyed on her known
/// playable slot 0 (empathy {R1, Y1, G1}).
#[test]
fn delayed_play_clue_admits_full_rank2_union_on_focus() {
    let (table_state, static_data, team_knowledge, history, actions) =
        common::load_scenario_with_knowledge("delayed_play_clue", 1);
    let clue_action = &actions[0];

    // Build Alice's pre-clue POV. Slot 0 is known playable (r1y1g1), slots 1 and 3
    // are touched by the rank-2 clue (r2 and p2 are visible in the scenario).
    let alice_knowledge = team_knowledge.player(0).clone();
    let alice_pov = LightweightPlayerPOV::new(
        0,
        &alice_knowledge,
        &team_knowledge,
        &table_state,
        &static_data,
    );

    // Both techs must match the rank-2 clue to Alice.
    assert!(
        DirectPlayClue.matches_action(clue_action, &history, &alice_pov),
        "DirectPlayClue should match the rank-2 clue to Alice"
    );
    assert!(
        DelayedPlayClue.matches_action(clue_action, &history, &alice_pov),
        "DelayedPlayClue should match the rank-2 clue to Alice"
    );

    // Gather knowledge updates from both techs.
    let direct_play_updates = DirectPlayClue.knowledge_updates(clue_action, &history, &alice_pov);
    let delayed_play_updates = DelayedPlayClue.knowledge_updates(clue_action, &history, &alice_pov);

    // Apply both hypotheses into a single cohort.
    let mut alice_live = alice_knowledge.clone();
    let mut next_id = 0u32;
    alice_live.apply_cohort(
        0,
        vec![(0, direct_play_updates), (0, delayed_play_updates)],
        &mut next_id,
        &static_data.variant,
    );

    // NO_VARIANT id layout: suit_idx * 5 + (rank - 1).
    // R2=1, Y2=6, G2=11, B2=16, P2=21.
    let expected_rank2_union: u64 = (1 << 1) | (1 << 6) | (1 << 11) | (1 << 16) | (1 << 21);
    let focus_mask = alice_live
        .effective_inferred_mask(3, &static_data.variant)
        .as_bits();
    assert_eq!(
        focus_mask & expected_rank2_union,
        expected_rank2_union,
        "Alice's empathy on the focused slot (deck 3) should be the full \
         union of direct ({{B2, P2}}) and delayed ({{R2, Y2, G2}}) candidates; \
         got {focus_mask:025b}"
    );
}
