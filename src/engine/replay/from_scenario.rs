//! Helpers for constructing engine types from scenario JSON.
//!
//! Used by the snapshot binary, integration tests, and any tooling that needs to load a
//! scenario file into a full `KnowledgeAwareGameState`.

use crate::engine::knowledge::team_knowledge::TeamKnowledge;
use crate::engine::knowledge_aware_game_state::KnowledgeAwareGameState;
use crate::game::card::CardIdentityMask;
use crate::game::state::table_state_json::{
    ScenarioJson, build_from_scenario, parse_card, parse_clue_string, parse_empathy_mask,
};
use crate::game::variant::Variant;

/// Build a `TeamKnowledge` from a `ScenarioJson`, applying all `positive`/`negative` clue
/// constraints and `inferred` masks declared in the scenario.
#[must_use]
pub fn team_knowledge_from_scenario(scenario: &ScenarioJson, variant: &Variant) -> TeamKnowledge {
    let num_players = scenario.hands.len();
    let mut team_knowledge = TeamKnowledge::new(num_players);

    let player_indices: Vec<Vec<u8>> = {
        let mut next: u8 = 0;
        scenario
            .hands
            .iter()
            .map(|hand| {
                let hand_size = hand.len() as u8;
                let range: Vec<u8> = (next..next + hand_size).collect();
                next += hand_size;
                range
            })
            .collect()
    };

    for (p, indices) in player_indices.iter().enumerate() {
        let own_hand: u64 = indices.iter().fold(0u64, |acc, &i| acc | (1 << i));
        team_knowledge.player_mut(p).own_hand = own_hand;
    }

    // Record known identities of teammates' cards.
    for p in 0..num_players {
        for (other, (hand, indices)) in scenario.hands.iter().zip(player_indices.iter()).enumerate()
        {
            if other == p {
                continue;
            }
            let hand_size = hand.len();
            for (slot_pos, slot) in hand.iter().enumerate() {
                let deck_idx = indices[hand_size - 1 - slot_pos];
                let id = slot.id();
                if id != "x" {
                    team_knowledge
                        .player_mut(p)
                        .update_with_revealed_card(deck_idx, parse_card(id));
                }
            }
        }
    }

    // Apply inferred masks (convention-based knowledge about own or others' cards).
    for (p, player_hand) in scenario.hands.iter().enumerate() {
        if p >= num_players {
            break;
        }
        let indices = &player_indices[p];
        let hand_size = player_hand.len();
        for (slot_pos, slot) in player_hand.iter().enumerate() {
            if slot_pos >= hand_size {
                break;
            }
            if let Some(inferred_str) = slot.inferred() {
                if inferred_str != "x" {
                    let deck_idx = indices[hand_size - 1 - slot_pos];
                    let mask = parse_empathy_mask(inferred_str);
                    let emp = CardIdentityMask::from_bits(mask);
                    team_knowledge.player_mut(p).inferred_identities[deck_idx as usize] = Some(emp);
                    if emp.is_exactly_known() {
                        team_knowledge.player_mut(p).visible_cards |= 1u64 << deck_idx;
                    }
                }
            }
        }
    }

    // Apply positive/negative clue marks (narrows own-hand inferred empathy).
    for (p, player_hand) in scenario.hands.iter().enumerate() {
        if p >= num_players {
            break;
        }
        let indices = &player_indices[p];
        let hand_size = player_hand.len();
        for (slot_pos, slot) in player_hand.iter().enumerate() {
            let deck_idx = indices[hand_size - 1 - slot_pos] as usize;
            for clue_str in slot.positive() {
                let (ct, cv) = parse_clue_string(clue_str);
                let clue_mask = variant.empathy_by_clue(ct, cv as usize).as_bits();
                team_knowledge
                    .player_mut(p)
                    .narrow_inferred(deck_idx as u8, clue_mask, variant);
            }
            for clue_str in slot.negative() {
                let (ct, cv) = parse_clue_string(clue_str);
                let clue_mask = variant.empathy_by_clue(ct, cv as usize).as_bits();
                team_knowledge
                    .player_mut(p)
                    .narrow_inferred(deck_idx as u8, !clue_mask, variant);
            }
        }
    }

    team_knowledge
}

/// Build a `KnowledgeAwareGameState` from a `ScenarioJson`.
///
/// Deck indices in the resulting state follow the scenario format (0..N, sequential per player).
/// The `next_deck_index` is set to the total number of dealt cards.
#[must_use]
pub fn knowledge_aware_from_scenario(
    scenario: &ScenarioJson,
    variant: Variant,
) -> KnowledgeAwareGameState {
    let (table_state, static_data) = build_from_scenario(scenario, variant);
    let team_knowledge = team_knowledge_from_scenario(scenario, &static_data.variant);
    let next_deck_index: u8 = scenario.hands.iter().map(|h| h.len() as u8).sum();
    KnowledgeAwareGameState::from_parts(static_data, table_state, team_knowledge, next_deck_index)
}
