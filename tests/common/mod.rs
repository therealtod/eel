use eel::engine::knowledge::player_knowledge::PlayerKnowledge;
use eel::engine::knowledge::team_knowledge::TeamKnowledge;
use eel::game::card::CardIdentityMask;
use eel::game::state::table_state::TableState;
use eel::game::state::table_state_json::{
    ScenarioJson, build_from_scenario, parse_card, parse_empathy_mask, parse_scenario,
};
use eel::game::static_game_data::StaticGameData;
use eel::game::variant::test_variants::NO_VARIANT;
use std::path::PathBuf;
use std::sync::OnceLock;

static TRACING: OnceLock<()> = OnceLock::new();

#[allow(dead_code)]
pub fn init_tracing() {
    TRACING.get_or_init(|| {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                    tracing_subscriber::EnvFilter::new("eel::search=debug,eel::apply=debug")
                }),
            )
            .with_test_writer()
            .init();
    });
}

#[allow(dead_code)]
pub fn load_scenario(n: u32) -> (TableState, StaticGameData) {
    init_tracing();
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

#[allow(dead_code)]
pub fn team_knowledge_from_scenario(scenario: &ScenarioJson) -> TeamKnowledge {
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

    for p in 0..num_players {
        for (other, (hand, indices)) in scenario.hands.iter().zip(player_indices.iter()).enumerate()
        {
            if other == p {
                continue;
            }
            let hand_size = hand.len();
            for (slot_pos, card_str) in hand.iter().enumerate() {
                let deck_idx = indices[hand_size - 1 - slot_pos];
                if card_str != "x" {
                    team_knowledge
                        .player_mut(p)
                        .update_with_revealed_card(deck_idx, parse_card(card_str));
                }
            }
        }
    }

    for (p, inferred_hand) in scenario.inferred_identities.iter().enumerate() {
        if p >= num_players {
            break;
        }
        let indices = &player_indices[p];
        let hand_size = scenario.hands[p].len();
        for (slot_pos, entry) in inferred_hand.iter().enumerate() {
            if slot_pos >= hand_size {
                break;
            }
            if entry != "x" {
                let deck_idx = indices[hand_size - 1 - slot_pos];
                let mask = parse_empathy_mask(entry);
                let emp = CardIdentityMask::from_bits(mask);
                team_knowledge.player_mut(p).inferred_identities[deck_idx as usize] = Some(emp);
                if emp.is_exactly_known() {
                    team_knowledge.player_mut(p).visible_cards |= 1u64 << deck_idx;
                }
            }
        }
    }

    team_knowledge
}

#[allow(dead_code)]
pub fn load_scenario_with_knowledge(n: u32) -> (TableState, StaticGameData, TeamKnowledge) {
    init_tracing();
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

#[allow(dead_code)]
pub fn knowledge_for_hand(player_index: usize, deck_indices: &[u8]) -> PlayerKnowledge {
    let mut k = PlayerKnowledge::new(player_index);
    k.own_hand = deck_indices.iter().fold(0u64, |acc, &i| acc | (1 << i));
    k
}
