use crate::common;

use eel::engine::convention::convention_tech::ConventionTech;
use eel::engine::convention::hgroup::signal::Signal;
use eel::engine::convention::hgroup::tech::critical_save::RANK_CRITICAL_SAVE;
use eel::engine::convention::hgroup::tech::direct_play_clue::DirectPlayClue;
use eel::engine::convention::hgroup::tech::play_known_playable::PlayKnownPlayable;
use eel::engine::convention::hgroup::tech::simple_finesse::SimpleFinesse;
use eel::engine::knowledge::knowledge_update::{KnowledgeUpdate, PendingTrigger};
use eel::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
use eel::engine::knowledge::player_knowledge::PlayerKnowledge;
use eel::game::action::game_action::GameAction;

#[test]
fn all_players_understand_simple_finesse_semantics() {
    let (table_state, static_data, team_knowledge, history, actions) =
        common::load_scenario_with_knowledge("simple_finesse", 1);
    let finesse_action = &actions[0];

    let alice_knowledge = team_knowledge.player(0).clone();
    let alice_pov = LightweightPlayerPOV::new(
        0,
        &alice_knowledge,
        &team_knowledge,
        &table_state,
        &static_data,
    );

    let generated_actions = SimpleFinesse.game_actions(&alice_pov);
    assert!(
        generated_actions.contains(finesse_action),
        "Alice should generate a rank-3 finesse clue to Cathy (deck 13 = r3, 1-away)"
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

    let bob_updates = SimpleFinesse.knowledge_updates(finesse_action, &history, &bob_pov);
    assert!(
        bob_updates.trigger.is_none(),
        "Bob's play obligation is unconditional: he can see all the information directly"
    );
    assert!(
        bob_updates.immediate.iter().any(|u| matches!(
            u,
            KnowledgeUpdate::NarrowPossibilities {
                card_deck_index: 9,
                ..
            }
        )),
        "Bob should have deck 9 narrowed to the connecting card identity (r2)"
    );
    assert!(
        bob_updates.immediate.iter().any(|u| matches!(
            u,
            KnowledgeUpdate::AddSignal {
                card_deck_index: 9,
                signal: Signal::Play { .. }
            }
        )),
        "Bob must receive a Play signal on deck 9 (r2, his finesse position)"
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

    let cathy_updates = SimpleFinesse.knowledge_updates(finesse_action, &history, &cathy_pov);

    const R3_MASK: u64 = 1u64 << 2;
    assert!(
        cathy_updates.immediate.iter().any(|u| matches!(u,
            KnowledgeUpdate::NarrowPossibilities { card_deck_index: 13, mask }
            if *mask == R3_MASK
        )),
        "Cathy's finesse hypothesis should pin deck 13 to r3 (the 1-away identity)"
    );

    assert!(
        DirectPlayClue.matches_action(finesse_action, &history, &cathy_pov),
        "DirectPlayClue should match for Cathy: her deck-13 empathy allows b3 (playable)"
    );
    let cathy_direct_play_updates =
        DirectPlayClue.knowledge_updates(finesse_action, &history, &cathy_pov);
    assert!(
        cathy_direct_play_updates
            .immediate
            .iter()
            .any(|u| matches!(u,
                KnowledgeUpdate::NarrowPossibilities { card_deck_index: 13, mask }
                if *mask == B3_MASK
            )),
        "DirectPlayClue should narrow deck 13 to b3 (immediately playable rank-3 card, blue stack = b2)"
    );
    assert!(
        cathy_direct_play_updates.trigger.is_none(),
        "DirectPlayClue produces an unconditional hypothesis"
    );

    match &cathy_updates.trigger {
        Some(PendingTrigger::BlindPlay {
            player,
            expected_card,
            ..
        }) => {
            assert_eq!(*player, 1, "trigger waits for Bob (player 1) to act");
            assert_eq!(
                *expected_card, 9,
                "confirms if Bob blind-plays deck 9 (r2); rejects otherwise, resolving b3-or-r3 ambiguity"
            );
        }
        _ => panic!(
            "expected a BlindPlay pending trigger pointing at Bob's finesse position (deck 9)"
        ),
    }
}

const R3_MASK: u64 = 1u64 << 2;
const B3_MASK: u64 = 1u64 << 17;

fn cathy_knowledge_after_finesse_clue()
-> (PlayerKnowledge, eel::game::static_game_data::StaticGameData) {
    let (table_state, static_data, team_knowledge, history, actions) =
        common::load_scenario_with_knowledge("simple_finesse", 1);
    let finesse_action = &actions[0];

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

    let finesse_hypothesis = SimpleFinesse.knowledge_updates(finesse_action, &history, &cathy_pov);
    let direct_play_hypothesis =
        DirectPlayClue.knowledge_updates(finesse_action, &history, &cathy_pov);

    let mut knowledge_live = cathy_knowledge.clone();
    let mut next_id = 0u32;
    knowledge_live.apply_cohort(
        0,
        vec![(0, finesse_hypothesis), (0, direct_play_hypothesis)],
        &mut next_id,
        &static_data.variant,
    );
    (knowledge_live, static_data)
}

#[test]
fn cathy_finesse_hypothesis_confirms_when_bob_blind_plays() {
    let (mut knowledge, static_data) = cathy_knowledge_after_finesse_clue();

    let pre = knowledge
        .effective_inferred_mask(13, &static_data.variant)
        .as_bits();
    assert_eq!(
        pre & (R3_MASK | B3_MASK),
        R3_MASK | B3_MASK,
        "before resolution deck 13 should admit both r3 and b3"
    );

    knowledge.resolve_pending(
        1,
        &GameAction::Play {
            player_index: 1,
            card_deck_index: 9,
            turn: 1,
        },
        None,
        &static_data.variant,
    );

    assert!(
        knowledge.hypotheses.is_empty(),
        "after Bob blind-plays all hypotheses should be resolved"
    );
    let post = knowledge
        .effective_inferred_mask(13, &static_data.variant)
        .as_bits();
    assert_eq!(
        post & (R3_MASK | B3_MASK),
        R3_MASK,
        "after Bob blind-plays deck 9 (r2), deck 13 should be pinned to r3 only"
    );
}

#[test]
fn cathy_finesse_hypothesis_rejects_when_bob_does_not_blind_play() {
    let (mut knowledge, static_data) = cathy_knowledge_after_finesse_clue();

    knowledge.resolve_pending(
        1,
        &GameAction::Discard {
            player_index: 1,
            card_deck_index: 5,
            turn: 1,
        },
        None,
        &static_data.variant,
    );

    assert_eq!(
        knowledge.hypotheses.len(),
        1,
        "only the finesse hypothesis is removed; the direct-play (b3) sibling remains"
    );
    let post = knowledge
        .effective_inferred_mask(13, &static_data.variant)
        .as_bits();
    assert_eq!(
        post & (R3_MASK | B3_MASK),
        B3_MASK,
        "after Bob discards, only the direct-play interpretation (b3) should remain"
    );
}

#[test]
fn rank_3_clue_on_chop_three_interpretations_finesse_excluded_by_critical_save() {
    const G3_MASK: u64 = 1u64 << 12;

    let (table_state, static_data, team_knowledge, history, actions) =
        common::load_scenario_with_knowledge("simple_finesse", 2);
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
        RANK_CRITICAL_SAVE
            .game_actions(&alice_pov)
            .contains(clue_action),
        "Alice should generate a rank-3 critical save to Cathy (deck 10 = g3, critical chop)"
    );
    assert!(
        !SimpleFinesse.game_actions(&alice_pov).contains(clue_action),
        "Alice should NOT treat rank-3 to Cathy as a finesse (g3 is 3-away, not a finesse target)"
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
    assert!(
        SimpleFinesse
            .knowledge_updates(clue_action, &history, &bob_pov)
            .is_empty(),
        "Bob sees g3 on Cathy's chop (3-away): no finesse, SimpleFinesse returns empty for Bob"
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

    let finesse_hypothesis = SimpleFinesse.knowledge_updates(clue_action, &history, &cathy_pov);
    let critical_save_hypothesis =
        RANK_CRITICAL_SAVE.knowledge_updates(clue_action, &history, &cathy_pov);
    let direct_play_hypothesis =
        DirectPlayClue.knowledge_updates(clue_action, &history, &cathy_pov);

    assert!(
        finesse_hypothesis.trigger.is_some(),
        "SimpleFinesse should give Cathy a provisional hypothesis (trigger on Bob's blind-play)"
    );
    assert!(
        !critical_save_hypothesis.is_empty(),
        "RankCriticalSave should give Cathy an unconditional hypothesis pinning deck 10 to g3"
    );

    let mut cathy_live = cathy_knowledge.clone();
    let mut next_id = 0u32;
    cathy_live.apply_cohort(
        0,
        vec![
            (0, finesse_hypothesis),
            (0, critical_save_hypothesis),
            (0, direct_play_hypothesis),
        ],
        &mut next_id,
        &static_data.variant,
    );

    let pre_mask = cathy_live
        .effective_inferred_mask(10, &static_data.variant)
        .as_bits();
    assert_eq!(
        pre_mask & (R3_MASK | B3_MASK | G3_MASK),
        R3_MASK | B3_MASK | G3_MASK,
        "before Bob acts, deck 10 admits r3 (finesse), b3 (direct play), and g3 (critical save)"
    );

    cathy_live.resolve_pending(
        1,
        &GameAction::Discard {
            player_index: 1,
            card_deck_index: 5,
            turn: 1,
        },
        None,
        &static_data.variant,
    );

    assert_eq!(
        cathy_live.hypotheses.len(),
        2,
        "after Bob discards, finesse is rejected; the two unconditional hypotheses (g3 save and b3 play) remain"
    );
    let post_mask = cathy_live
        .effective_inferred_mask(10, &static_data.variant)
        .as_bits();
    assert_eq!(
        post_mask & (R3_MASK | B3_MASK | G3_MASK),
        B3_MASK | G3_MASK,
        "after Bob discards, only direct-play (b3) and critical-save (g3) interpretations survive"
    );

    let mut team_knowledge_post = team_knowledge.clone();
    *team_knowledge_post.player_mut(2) = cathy_live.clone();
    let cathy_post_pov = LightweightPlayerPOV::new(
        2,
        &cathy_live,
        &team_knowledge_post,
        &cathy_table_state,
        &static_data,
    );
    assert!(
        !PlayKnownPlayable
            .game_actions(&cathy_post_pov)
            .iter()
            .any(|a| matches!(
                a,
                GameAction::Play {
                    card_deck_index: 10,
                    ..
                }
            )),
        "Cathy should not play deck 10: it could be g3 (not playable), making the play unsafe"
    );
}
