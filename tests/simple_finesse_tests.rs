mod common;

use eel::engine::convention::convention_tech::ConventionTech;
use eel::engine::convention::hgroup::signal::Signal;
use eel::engine::convention::hgroup::tech::simple_finesse::SimpleFinesse;
use eel::engine::game_state_snapshot::GameStateSnapshot;
use eel::engine::knowledge::knowledge_update::{Hypothesis, KnowledgeUpdate, PendingTrigger};
use eel::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
use eel::engine::knowledge::player_knowledge::PlayerKnowledge;
use eel::game::action::game_action::GameAction;
use eel::game::clue::Clue;
use eel::game::clue_type::ClueType;
use smallvec::smallvec;

// Scenario 16: 3p, stacks=[r1,b2], player 0 on turn
// p1=["r2","b3","y4","b1","r3"] → slot1=deck9=r2 (finesse position, id=1, playable)
// p2=["b4","r3","p2","p2","r1"] → slot2=deck13=r3 (id=2, 1-away; connecting=r2)
// p1 plays before p2; p1 finesse position = r2 = connecting card for r3
// → SimpleFinesse generates a clue to p2 focusing r3
#[test]
fn all_players_understand_simple_finesse_semantics() {
    let (table_state, static_data, team_knowledge) = common::load_scenario_with_knowledge(16);

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

    let actions = SimpleFinesse.game_actions(&alice_pov);
    let finesse_action = GameAction::Clue {
        player_index: 2,
        touched_card_deck_indexes: smallvec![13],
        clue: Clue {
            clue_type: ClueType::Rank,
            clue_value: 3,
        },
        turn: 0,
    };
    assert!(
        actions.contains(&finesse_action),
        "Alice should generate a rank-3 finesse clue to Cathy (deck 13 = r3, 1-away)"
    );

    // ── Part 2: Build pre-clue history snapshot ──────────────────────────────────────────────────
    // The snapshot captures the game state before Alice gives the clue.
    let snapshot = GameStateSnapshot::new(table_state.clone(), team_knowledge.clone());
    let history = vec![snapshot];

    // ── Part 3: Bob (player 1) receives an unconditional Play signal on his finesse position ─────
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

    let bob_updates = SimpleFinesse.knowledge_updates(&finesse_action, &history, &bob_pov);
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

    // ── Part 4: Cathy (player 2) holds two competing interpretations of the rank-3 clue ─────────
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

    let cathy_updates = SimpleFinesse.knowledge_updates(&finesse_action, &history, &cathy_pov);

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

/// Build Cathy's live `PlayerKnowledge` for scenario 16 after Alice gives a rank-3 finesse clue.
///
/// Returns a knowledge state with two live hypotheses in cohort 0:
///   - finesse (r3, provisional on Bob blind-playing deck 9): from `SimpleFinesse`
///   - direct play (b3, unconditional): stand-in for what `DelayedPlayClue` would contribute
///
/// While both are live the effective mask for deck 13 is r3 | b3.
fn cathy_knowledge_after_finesse_clue()
-> (PlayerKnowledge, eel::game::static_game_data::StaticGameData) {
    let (table_state, static_data, team_knowledge) = common::load_scenario_with_knowledge(16);

    let snapshot = GameStateSnapshot::new(table_state.clone(), team_knowledge.clone());
    let history = vec![snapshot];

    let finesse_action = GameAction::Clue {
        player_index: 2,
        touched_card_deck_indexes: smallvec![13],
        clue: Clue {
            clue_type: ClueType::Rank,
            clue_value: 3,
        },
        turn: 0,
    };

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

    let finesse_hypothesis = SimpleFinesse.knowledge_updates(&finesse_action, &history, &cathy_pov);
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

// Scenario 16: Bob discards instead → finesse rejected, only the direct-play (b3) interpretation remains
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

// Scenario 17: 3p, stacks=[r1,b2]
// p1=["r3","b3","y4","b1","g5"] → slot1=deck9=r3 (id=2, 1-away; NOT a connecting card for any 1-away focus)
// p2=["r2","r3","p2","p2","y2"] → chop=deck10=y2 (1-away), slot1=deck14=r2 (playable)
// p1 finesse position = r3 (id=2). For finesse: need focus with connecting=r3, i.e. focus=r4 (2-away). No valid finesse.
#[test]
#[ignore]
fn simple_finesse_returns_empty_when_no_valid_finesse_setup() {
    let (table_state, static_data, team_knowledge) = common::load_scenario_with_knowledge(17);
    let knowledge = team_knowledge.player(0).clone();
    let pov = LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

    // No player has a 1-away card whose connecting card is on p1's finesse position (r3)
    // (r4 would need r3 as connecting, but no one has r4 as a 1-away focus)
    let actions = SimpleFinesse.game_actions(&pov);

    assert!(
        !actions.iter().any(|a| matches!(a, GameAction::Clue { player_index: 2, .. }
            if matches!(a, GameAction::Clue { clue, .. } if clue.clue_type == ClueType::Color && clue.clue_value == 0))),
        "should not generate a red finesse clue to player 2"
    );
}

// Scenario 16: matches_action true for a red clue to p2 focusing r3 (finesse on p1's r2)
#[test]
#[ignore]
fn simple_finesse_matches_action_true_for_valid_finesse_clue() {
    let (table_state, static_data, team_knowledge) = common::load_scenario_with_knowledge(16);
    let knowledge = team_knowledge.player(0).clone();
    let pov = LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

    // Red clue to p2 touching deck13=r3 (focus, 1-away; connecting=r2 on p1's finesse position)
    let action = GameAction::Clue {
        player_index: 2,
        touched_card_deck_indexes: smallvec::smallvec![13],
        clue: Clue {
            clue_type: ClueType::Color,
            clue_value: 0,
        }, // red
        turn: 0,
    };
    assert!(SimpleFinesse.matches_action(&action, &[], &pov));
}

// Scenario 16: matches_action false for a clue whose focus is directly playable (not 1-away)
#[test]
#[ignore]
fn simple_finesse_matches_action_false_when_focus_not_1_away() {
    let (table_state, static_data, team_knowledge) = common::load_scenario_with_knowledge(16);
    let knowledge = team_knowledge.player(0).clone();
    let pov = LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

    // Blue clue to p2 touching deck11=p2 (not 1-away)
    let action = GameAction::Clue {
        player_index: 2,
        touched_card_deck_indexes: smallvec::smallvec![14],
        clue: Clue {
            clue_type: ClueType::Color,
            clue_value: 2,
        }, // blue (b4 at deck14)
        turn: 0,
    };
    assert!(!SimpleFinesse.matches_action(&action, &[], &pov));
}
