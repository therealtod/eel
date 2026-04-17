use std::path::PathBuf;
use eel::engine::knowledge::player_knowledge_state::PlayerKnowledgeState;
use eel::engine::knowledge::team_knowledge::TeamKnowledge;
use eel::game::state::table_state::TableState;
use eel::game::state::table_state_json::{build_from_scenario, parse_card, parse_scenario, ScenarioJson};
use eel::game::static_game_data::StaticGameData;
use eel::game::variant::test_variants::NO_VARIANT;

/// Load a scenario by number and return `(TableState, StaticGameData)`.
///
/// Scenarios live at `tests/scenarios/scenario{n}/table_state.json`.
pub fn load_scenario(n: u32) -> (TableState, StaticGameData) {
    let path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "tests",
        "scenarios",
        &format!("scenario{n}"),
        "table_state.json",
    ]
    .iter()
    .collect();

    let json = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read scenario {n}: {e}"));
    let scenario = parse_scenario(&json);
    build_from_scenario(&scenario, NO_VARIANT)
}

/// Build a `TeamKnowledge` from a `ScenarioJson`.
///
/// Deck indices are assigned sequentially (player 0 gets 0..hand_size, etc.).
/// Each player's `own_hand` covers their deck indices; `visible_cards` and
/// `empathy` are populated for all non-"x" cards held by other players.
#[allow(dead_code)]
pub fn team_knowledge_from_scenario(scenario: &ScenarioJson) -> TeamKnowledge {
    let num_players = scenario.hands.len();
    let mut team_knowledge = TeamKnowledge::new(num_players);

    // Compute the deck-index range for each player.
    let player_indices: Vec<Vec<u8>> = {
        let mut next: u8 = 0;
        scenario.hands.iter().map(|hand| {
            let range: Vec<u8> = (next..next + hand.len() as u8).collect();
            next += hand.len() as u8;
            range
        }).collect()
    };

    // Set own_hand for every player.
    for (p, indices) in player_indices.iter().enumerate() {
        let own_hand: u64 = indices.iter().fold(0u64, |acc, &i| acc | (1 << i));
        team_knowledge.player_mut(p).own_hand = own_hand;
    }

    // Populate visible_cards and empathy: player P sees other players' revealed cards.
    for p in 0..num_players {
        for (other, (hand, indices)) in scenario.hands.iter().zip(player_indices.iter()).enumerate() {
            if other == p { continue; }
            for (card_str, &idx) in hand.iter().zip(indices.iter()) {
                if card_str != "x" {
                    team_knowledge.player_mut(p)
                        .update_with_revealed_card(idx, parse_card(card_str));
                }
            }
        }
    }

    team_knowledge
}

/// Load a scenario and also build the corresponding `TeamKnowledge`.
#[allow(dead_code)]
pub fn load_scenario_with_knowledge(n: u32) -> (TableState, StaticGameData, TeamKnowledge) {
    let path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "tests",
        "scenarios",
        &format!("scenario{n}"),
        "table_state.json",
    ]
    .iter()
    .collect();

    let json = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read scenario {n}: {e}"));
    let scenario = parse_scenario(&json);
    let team_knowledge = team_knowledge_from_scenario(&scenario);
    let (table_state, static_data) = build_from_scenario(&scenario, NO_VARIANT);
    (table_state, static_data, team_knowledge)
}

/// Convenience: build a `PlayerKnowledgeState` whose `own_hand` covers the given deck indices.
#[allow(dead_code)]
pub fn knowledge_for_hand(player_index: usize, deck_indices: &[u8]) -> PlayerKnowledgeState {
    let mut k = PlayerKnowledgeState::new(player_index);
    k.own_hand = deck_indices.iter().fold(0u64, |acc, &i| acc | (1 << i));
    k
}
