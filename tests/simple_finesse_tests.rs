mod common;

use eel::engine::convention::convention_tech::ConventionTech;
use eel::engine::convention::hgroup::signal::Signal;
use eel::engine::convention::hgroup::tech::critical_save::RankCriticalSave;
use eel::engine::convention::hgroup::tech::play_known_playable::PlayKnownPlayable;
use eel::engine::convention::hgroup::tech::simple_finesse::SimpleFinesse;
use eel::engine::knowledge::knowledge_update::{Hypothesis, KnowledgeUpdate, PendingTrigger};
use eel::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
use eel::engine::knowledge::player_knowledge::PlayerKnowledge;
use eel::game::action::game_action::GameAction;

// Scenario 1: 3p, stacks=[r1,b2], player 0 on turn
// p1=["r2","b3","y4","b1","r3"] → slot1=deck9=r2 (finesse position, id=1, playable)
// p2=["b4","r3","p2","p2","r1"] → slot2=deck13=r3 (id=2, 1-away; connecting=r2)
// p1 plays before p2; p1 finesse position = r2 = connecting card for r3
// → SimpleFinesse generates a clue to p2 focusing r3
#[test]
fn all_players_understand_simple_finesse_semantics() {
    let (table_state, static_data, team_knowledge, history, actions) =
        common::load_scenario_with_knowledge(1);
    // history[0] = pre-clue snapshot; actions[0] = Alice's rank-3 clue to Cathy (deck 13)
    let finesse_action = &actions[0];

    // ── Part 1: Alice (player 0) can generate the rank-3 finesse clue to Cathy ─────────────────
    // Rank 3 touches only deck 13 (r3) in Cathy's hand; r3 is 1-away (red stack at r1, r2 not
    // yet played), and Bob's finesse position (deck 9 = r2) is the connecting card.
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

    // ── Part 2: Bob (player 1) receives an unconditional Play signal on his finesse position ─────
    // Bob is a third-party observer who sees both hands. He recognises that Cathy's deck 13 is
    // r3 (1-away) and that his own deck 9 (r2) is the connecting card, so he must blind-play it.
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

    // ── Part 3: Cathy (player 2) holds two competing interpretations of the rank-3 clue ─────────
    // From Cathy's POV the clue is ambiguous:
    //   a) Direct play clue → focused card is b3 (blue 3, directly playable since blue stack = b2)
    //      (contributed by DelayedPlayClue / other tech, not tested here)
    //   b) Finesse → focused card is r3 (red 3, 1-away), confirmed only if Bob blind-plays r2
    //      (contributed by SimpleFinesse as a *provisional* hypothesis)
    // The ambiguity resolves on Bob's next turn: blind-play of deck 9 confirms (b) and prunes (a);
    // any other Bob action rejects (b) and leaves only (a).
    //
    // knowledge_updates uses active_player_index to decide whether the observer is the receiver;
    // set it to 2 so Cathy enters the receiver branch.
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

    // The finesse hypothesis pins the focused card (deck 13) to r3 (id = 2, mask = 1 << 2).
    const R3_MASK: u64 = 1u64 << 2; // R3 = red suit offset 0, rank 3, id = 0 + 3 - 1 = 2
    assert!(
        cathy_updates.immediate.iter().any(|u| matches!(u,
            KnowledgeUpdate::NarrowPossibilities { card_deck_index: 13, mask }
            if *mask == R3_MASK
        )),
        "Cathy's finesse hypothesis should pin deck 13 to r3 (the 1-away identity)"
    );

    // The hypothesis is provisional: it resolves when Bob takes his next action.
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

// Deck-13 constants used by the resolution tests below.
const R3_MASK: u64 = 1u64 << 2; // R3 = red offset 0, rank 3, id = 2
const B3_MASK: u64 = 1u64 << 17; // B3 = blue offset 15, rank 3, id = 17

/// Build Cathy's live `PlayerKnowledge` for scenario 1 after Alice gives a rank-3 finesse clue.
///
/// Returns a knowledge state with two live hypotheses in cohort 0:
///   - finesse (r3, provisional on Bob blind-playing deck 9): from `SimpleFinesse`
///   - direct play (b3, unconditional): stand-in for what `DelayedPlayClue` would contribute
///
/// While both are live the effective mask for deck 13 is r3 | b3.
fn cathy_knowledge_after_finesse_clue()
-> (PlayerKnowledge, eel::game::static_game_data::StaticGameData) {
    let (table_state, static_data, team_knowledge, history, actions) =
        common::load_scenario_with_knowledge(1);
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

    let finesse_hypothesis =
        SimpleFinesse.knowledge_updates(finesse_action, &history, &cathy_pov);
    let b3_direct_play = Hypothesis::unconditional(vec![KnowledgeUpdate::NarrowPossibilities {
        card_deck_index: 13,
        mask: B3_MASK,
    }]);

    let mut knowledge_live = cathy_knowledge.clone();
    let mut next_id = 0u32;
    knowledge_live.apply_cohort(
        0,
        vec![finesse_hypothesis, b3_direct_play],
        &mut next_id,
        &static_data.variant,
    );
    (knowledge_live, static_data)
}

// Scenario 16: Bob blind-plays deck 9 → finesse confirmed, deck 13 pinned to r3
#[test]
fn cathy_finesse_hypothesis_confirms_when_bob_blind_plays() {
    let (mut knowledge, static_data) = cathy_knowledge_after_finesse_clue();

    // Before resolution: both r3 and b3 are live.
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

// Scenario 2: Bob discards instead → finesse rejected, only the direct-play (b3) interpretation remains
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
        &static_data.variant,
    );

    // Only the provisional finesse is dropped; the unconditional b3 sibling stays.
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

// Scenario 2: 3p, stacks=[r1,0,0,b2,0], discard=[g3], player 0 on turn
// p1=["r2","b3","y4","b1","r3"] → slot1=deck9=r2 (finesse position, playable), slot5=deck5=r3 (chop)
// p2=["b4","b4","p2","p2","g3"] → chop=deck10=g3 (critical, rank-3, 3-away from green stack)
//
// Alice's rank-3 clue to Cathy is a critical save on g3 (chop). From Cathy's POV it is ambiguous:
//   a) critical save → deck 10 is g3 (unconditional, from RankCriticalSave)
//   b) finesse → deck 10 is r3, confirmed if Bob blind-plays r2/deck9 (provisional, SimpleFinesse)
//   c) direct play → deck 10 is b3, playable since blue stack = b2 (unconditional stand-in)
// Bob sees g3 on chop (3-away, not a finesse target), so SimpleFinesse is empty for him.
// After Bob discards instead of blind-playing, finesse is rejected; b3 | g3 remain.
// Cathy cannot safely play deck 10 because g3 is not playable.
#[test]
fn rank_3_clue_on_chop_three_interpretations_finesse_excluded_by_critical_save() {
    const G3_MASK: u64 = 1u64 << 12; // G3 = green offset 10, rank 3, id = 12

    let (table_state, static_data, team_knowledge, history, actions) =
        common::load_scenario_with_knowledge(2);
    let clue_action = &actions[0];

    // ── Part 1: Alice generates a rank-3 critical save, not a finesse ─────────────────────────
    let alice_knowledge = team_knowledge.player(0).clone();
    let alice_pov = LightweightPlayerPOV::new(
        0,
        &alice_knowledge,
        &team_knowledge,
        &table_state,
        &static_data,
    );
    assert!(
        RankCriticalSave.game_actions(&alice_pov).contains(clue_action),
        "Alice should generate a rank-3 critical save to Cathy (deck 10 = g3, critical chop)"
    );
    assert!(
        !SimpleFinesse.game_actions(&alice_pov).contains(clue_action),
        "Alice should NOT treat rank-3 to Cathy as a finesse (g3 is 3-away, not a finesse target)"
    );

    // ── Part 2: Bob (third-party) gets no blind-play signal ───────────────────────────────────
    // From Bob's POV: focus = deck 10 = g3 (away=3), not a finesse setup.
    // The critical-save interpretation makes it clear to him: no finesse, no blind-play needed.
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

    // ── Part 3: Cathy holds three live interpretations ────────────────────────────────────────
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
        RankCriticalSave.knowledge_updates(clue_action, &history, &cathy_pov);
    let direct_play_b3 = Hypothesis::unconditional(vec![KnowledgeUpdate::NarrowPossibilities {
        card_deck_index: 10,
        mask: B3_MASK,
    }]);

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
        vec![finesse_hypothesis, critical_save_hypothesis, direct_play_b3],
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

    // ── Part 4: Bob discards → finesse rejected; g3 and b3 interpretations remain ──────────────
    cathy_live.resolve_pending(
        1,
        &GameAction::Discard {
            player_index: 1,
            card_deck_index: 5,
            turn: 1,
        },
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

    // ── Part 5: Cathy does not play deck 10 (b3 playable, g3 not) ───────────────────────────
    // PlayKnownPlayable requires ALL possibilities to be playable. b3 is playable (blue=b2),
    // but g3 is not (green stack empty). So no play is generated for deck 10.
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
            .any(|a| matches!(a, GameAction::Play { card_deck_index: 10, .. })),
        "Cathy should not play deck 10: it could be g3 (not playable), making the play unsafe"
    );
}
