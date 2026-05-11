mod common;

use eel::engine::convention::convention_tech::ConventionTech;
use eel::engine::convention::hgroup::tech::delayed_play_clue::DelayedPlayClue;
use eel::engine::convention::hgroup::tech::simple_prompt::SimplePrompt;
use eel::engine::knowledge::knowledge_update::{Hypothesis, KnowledgeUpdate};
use eel::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;

// Scenario 3: 3p, stacks=[r1r2r3,_,_,b1b2,p1p2], Alice (p0) on turn.
// Deck layout (oldest→newest within each hand):
//   Alice (p0): decks 0-4; deck 4 = p3 (purple clue, convention-inferred p3)
//   Bob   (p1): decks 5-9;
//     deck 9=y4(+4), deck 8=b3(+3), deck 7=r3(+3), deck 6=p3(+3), deck 5=r2
//   Cathy (p2): decks 10-14;
//     deck 14=b4 (slot 1, newest), deck 13=p4, ..., deck 10=g3 (chop)
// Prior action: Alice gives rank-4 to Cathy, touching [14,13]; focus=deck 14 (b4, away=1).
// Connecting card: b3 (id 17). Bob's deck 9 (y4, rank-4 empathy) excludes b3; deck 8 (rank-3
// empathy = {r3,y3,g3,b3,p3}) is the first empathy-compatible card and IS the connecting card.

const B3_MASK: u64 = 1u64 << 17; // B3: blue offset 15, rank 3, id 17
const B4_MASK: u64 = 1u64 << 18; // B4: blue offset 15, rank 4, id 18
const R4_MASK: u64 = 1u64 << 3;  // R4: red offset 0,  rank 4, id  3
const P4_MASK: u64 = 1u64 << 23; // P4: purple offset 20, rank 4, id 23

#[test]
fn all_players_understand_simple_prompt_semantics() {
    let (table_state, static_data, team_knowledge, history, actions) =
        common::load_scenario_with_knowledge("simple_prompt", 1);
    let clue_action = &actions[0];

    // ── Part 1: Alice generates the rank-4 prompt clue to Cathy ──────────────────
    // Focus = deck 14 (b4, away=1 from blue stack b2). Connecting card = b3 (deck 8 in Bob's
    // hand). Bob's slot-1 card (deck 9 = y4, rank-4 empathy) is skipped because rank-4 does not
    // include b3; deck 8 (rank-3 empathy) is the first empathy-compatible card.
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

    // ── Part 2: Bob infers b3 on his first rank-3-clued slot (deck 8) ────────────
    // From Bob's POV: Alice's rank-4 clue focuses deck 14 (b4, away=1). Deck 9 (rank-4 empathy)
    // is not compatible with b3 and is skipped. Deck 8 (rank-3 empathy includes b3) is the
    // first compatible slot, so Bob gets an unconditional NarrowPossibilities update there.
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

    // ── Part 3: Cathy holds three competing interpretations of the rank-4 clue ───
    // - SimplePrompt:    deck 14 is b4 (b3 prompt on Bob)               — unconditional
    // - DelayedPlayClue: deck 14 is p4 (p3 known in Alice's hand deck 4) — unconditional
    // - Direct play:     deck 14 is r4 (r4 directly playable, red=r3)   — stand-in
    // Effective mask before any resolution must admit all three identities.
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
        vec![prompt_hypothesis, delayed_hypothesis, direct_play_r4],
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
