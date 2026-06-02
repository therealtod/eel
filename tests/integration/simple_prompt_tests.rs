use crate::common;

use eel::engine::convention::convention_tech::ConventionTech;
use eel::engine::convention::hgroup::tech::delayed_play_clue::DelayedPlayClue;
use eel::engine::convention::hgroup::tech::simple_prompt::SimplePrompt;
use eel::engine::knowledge::knowledge_update::{Hypothesis, KnowledgeUpdate};
use eel::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;

const B3_MASK: u64 = 1u64 << 17;
const B4_MASK: u64 = 1u64 << 18;
const R4_MASK: u64 = 1u64 << 3;
const P4_MASK: u64 = 1u64 << 23;

#[test]
fn all_players_understand_simple_prompt_semantics() {
    let (table_state, static_data, team_knowledge, history, actions) =
        common::load_scenario_with_knowledge("simple_prompt", 1);
    let clue_action = &actions[0];

    let alice_knowledge = team_knowledge.player(0).clone();
    let alice_pov = LightweightPlayerPOV::new(
        0,
        &alice_knowledge,
        &team_knowledge,
        &table_state,
        &static_data,
    );
    assert!(
        SimplePrompt.game_actions(&alice_pov).contains(clue_action),
        "Alice should generate the rank-4 prompt clue to Cathy (b4 focus, b3 prompt on Bob)"
    );

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

    let bob_updates = SimplePrompt.knowledge_updates(clue_action, &history, &bob_pov);
    assert!(
        bob_updates.trigger.is_none(),
        "Bob's update is unconditional: he can see all the information directly"
    );
    assert!(
        bob_updates.immediate.iter().any(|u| matches!(
            u,
            KnowledgeUpdate::NarrowPossibilities { card_deck_index: 8, mask }
            if *mask == B3_MASK
        )),
        "Bob should have deck 8 narrowed to b3 (the connecting card for b4)"
    );

    let mut cathy_table_state = table_state.clone();
    cathy_table_state.active_player_index = 2;
    let cathy_knowledge = team_knowledge.player(2).clone();
    let cathy_pov = LightweightPlayerPOV::new(
        2,
        &cathy_knowledge,
        &team_knowledge,
        &cathy_table_state,
        &static_data,
    );

    let prompt_hypothesis = SimplePrompt.knowledge_updates(clue_action, &history, &cathy_pov);
    let delayed_hypothesis = DelayedPlayClue.knowledge_updates(clue_action, &history, &cathy_pov);
    let direct_play_r4 = Hypothesis::unconditional(vec![KnowledgeUpdate::NarrowPossibilities {
        card_deck_index: 14,
        mask: R4_MASK,
    }]);

    assert!(
        prompt_hypothesis.immediate.iter().any(|u| matches!(u,
            KnowledgeUpdate::NarrowPossibilities { card_deck_index: 14, mask }
            if mask & B4_MASK != 0
        )),
        "SimplePrompt should include b4 in Cathy's focus-card hypothesis (b3 prompt on Bob)"
    );
    assert!(
        delayed_hypothesis.immediate.iter().any(|u| matches!(u,
            KnowledgeUpdate::NarrowPossibilities { card_deck_index: 14, mask }
            if mask & P4_MASK != 0
        )),
        "DelayedPlayClue should include p4 in Cathy's hypothesis (Alice's p3 globally known)"
    );

    let mut cathy_live = cathy_knowledge.clone();
    let mut next_id = 0u32;
    cathy_live.apply_cohort(
        0,
        vec![
            (0, prompt_hypothesis),
            (0, delayed_hypothesis),
            (0, direct_play_r4),
        ],
        &mut next_id,
        &static_data.variant,
    );

    let effective = cathy_live
        .effective_inferred_mask(14, &static_data.variant)
        .as_bits();
    assert_eq!(
        effective & (R4_MASK | B4_MASK | P4_MASK),
        R4_MASK | B4_MASK | P4_MASK,
        "Cathy's deck 14 should admit r4 (direct play), b4 (prompt), and p4 (delayed play)"
    );
}
