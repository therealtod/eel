use std::path::PathBuf;
use eel::game::state::table_state::TableState;
use eel::game::state::table_state_json::{build_from_scenario, parse_scenario};
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
