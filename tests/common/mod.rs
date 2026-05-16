use eel::engine::game_state_snapshot::GameStateSnapshot;
use eel::engine::knowledge::player_knowledge::PlayerKnowledge;
use eel::engine::knowledge::team_knowledge::TeamKnowledge;
use eel::engine::replay::from_scenario::team_knowledge_from_scenario as lib_team_knowledge_from_scenario;
use eel::game::action::game_action::GameAction;
use eel::game::state::table_state::TableState;
use eel::game::state::table_state_json::{
    ScenarioJson, build_from_scenario, build_game_actions, parse_scenario,
};
use eel::game::static_game_data::StaticGameData;
use eel::game::variant::Variant;
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
            .with_ansi(false)
            .with_test_writer()
            .init();
    });
}

#[allow(dead_code)]
pub fn load_scenario(tech: &str, scenario: u32) -> (TableState, StaticGameData) {
    init_tracing();
    let path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "tests",
        "scenarios",
        tech,
        &format!("{scenario}"),
        "table_state.json",
    ]
    .iter()
    .collect();

    let json = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {tech} scenario {scenario}: {e}"));
    let scenario = parse_scenario(&json);
    build_from_scenario(&scenario, NO_VARIANT)
}

#[allow(dead_code)]
pub fn team_knowledge_from_scenario(scenario: &ScenarioJson, variant: &Variant) -> TeamKnowledge {
    lib_team_knowledge_from_scenario(scenario, variant)
}

/// Apply a `GameAction` to a `TableState` at the table-state level only (no convention
/// knowledge propagation). Used to build intermediate history snapshots when replaying
/// `prior_actions`.
fn apply_action_for_history(
    ts: &mut TableState,
    action: &GameAction,
    static_data: &StaticGameData,
) {
    match action {
        GameAction::Clue {
            player_index,
            touched_card_deck_indexes,
            clue,
            ..
        } => {
            ts.update_with_clue_action(
                touched_card_deck_indexes.clone(),
                clue.clone(),
                *player_index,
                static_data,
            );
        }
        GameAction::Play {
            card_deck_index, ..
        } => {
            ts.update_with_play_action(*card_deck_index);
        }
        GameAction::Discard {
            card_deck_index, ..
        } => {
            ts.update_with_discard_action(*card_deck_index, static_data);
        }
        GameAction::Draw { .. } => {}
    }
}

/// Load a scenario by a semantic name (e.g. `"search/play_known_playable"`).
///
/// Constructs the path as `tests/scenarios/{name}/table_state.json`. Otherwise identical
/// to [`load_scenario_with_knowledge`].
#[allow(dead_code)]
pub fn load_scenario_by_name_with_knowledge(
    name: &str,
) -> (
    TableState,
    StaticGameData,
    TeamKnowledge,
    Vec<GameStateSnapshot>,
    Vec<GameAction>,
) {
    init_tracing();
    let path: PathBuf = {
        let mut p: PathBuf = [env!("CARGO_MANIFEST_DIR"), "tests", "scenarios"]
            .iter()
            .collect();
        for component in name.split('/') {
            p.push(component);
        }
        p.push("table_state.json");
        p
    };

    let json = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read scenario '{name}': {e}"));
    let scenario = parse_scenario(&json);
    let (table_state, static_data) = build_from_scenario(&scenario, NO_VARIANT);
    let team_knowledge = team_knowledge_from_scenario(&scenario, &static_data.variant);
    let actions = build_game_actions(&scenario, &static_data.variant);

    let mut history = Vec::with_capacity(actions.len());
    let mut running_ts = table_state.clone();
    for action in &actions {
        history.push(GameStateSnapshot::new(
            running_ts.clone(),
            team_knowledge.clone(),
        ));
        apply_action_for_history(&mut running_ts, action, &static_data);
    }

    (table_state, static_data, team_knowledge, history, actions)
}

/// Load a scenario by a semantic name, returning only board state and static data.
#[allow(dead_code)]
pub fn load_scenario_by_name(name: &str) -> (TableState, StaticGameData) {
    let (ts, sd, _, _, _) = load_scenario_by_name_with_knowledge(name);
    (ts, sd)
}

/// Load a scenario with team knowledge, history, and parsed actions.
///
/// Returns the base scenario state (before `prior_actions` are applied), the team knowledge
/// derived from the scenario, a history of snapshots (one per prior action, each capturing
/// the state before that action), and the `prior_actions` as `GameAction`s.
///
/// Tests use `history[i]` as the snapshot before `actions[i]`, eliminating the need to
/// manually fabricate history or hardcode action structs.
#[allow(dead_code)]
pub fn load_scenario_with_knowledge(
    tech: &str,
    scenario: u32,
) -> (
    TableState,
    StaticGameData,
    TeamKnowledge,
    Vec<GameStateSnapshot>,
    Vec<GameAction>,
) {
    init_tracing();
    let path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "tests",
        "scenarios",
        tech,
        &format!("{scenario}"),
        "table_state.json",
    ]
    .iter()
    .collect();

    let json = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {tech} scenario {scenario}: {e}"));
    let scenario = parse_scenario(&json);
    let (table_state, static_data) = build_from_scenario(&scenario, NO_VARIANT);
    let team_knowledge = team_knowledge_from_scenario(&scenario, &static_data.variant);

    let actions = build_game_actions(&scenario, &static_data.variant);

    let mut history = Vec::with_capacity(actions.len());
    let mut running_ts = table_state.clone();
    for action in &actions {
        history.push(GameStateSnapshot::new(
            running_ts.clone(),
            team_knowledge.clone(),
        ));
        apply_action_for_history(&mut running_ts, action, &static_data);
    }

    (table_state, static_data, team_knowledge, history, actions)
}

#[allow(dead_code)]
pub fn knowledge_for_hand(player_index: usize, deck_indices: &[u8]) -> PlayerKnowledge {
    let mut k = PlayerKnowledge::new(player_index);
    k.own_hand = deck_indices.iter().fold(0u64, |acc, &i| acc | (1 << i));
    k
}
