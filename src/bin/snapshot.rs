/// Extract a self-contained scenario JSON from a hanab.live replay at a given turn.
///
/// The output can be placed under `tests/scenarios/search/<name>/table_state.json` and
/// used directly with the `search_regression.rs` harness.  When the position is
/// history-dependent (convention state accumulated over prior turns changes the engine
/// recommendation), the tool emits a warning and advises using the replay harness instead.
///
/// Usage:
///   cargo run --release --bin snapshot -- \
///       --replay logs/14/game_0042.json \
///       --turn 22 \
///       --out tests/scenarios/search/slowplay_at_22/table_state.json \
///       --description "Bot slowplayed g3 instead of saving b5 on chop"
use std::fs;
use std::path::PathBuf;

use clap::Parser;

use eel::engine::action_selection_strategy::ActionSelectionStrategy;
use eel::engine::convention::hgroup::h_group_convention_set::HGroupConventionSet;
use eel::engine::replay::from_scenario::knowledge_aware_from_scenario;
use eel::engine::replay::reconstruct::ReplayRunner;
use eel::engine::replay::snapshot::to_scenario_json;
use eel::engine::tree_action_selection_strategy::TreeActionSelectionStrategy;
use eel::external::hanablive::Game;
use eel::game::state::table_state_json::parse_scenario;

#[derive(Parser, Debug)]
#[command(about = "Extract a scenario JSON from a hanab.live replay at a given turn")]
struct Args {
    /// Path to the hanab.live JSON replay file.
    #[arg(long)]
    replay: PathBuf,

    /// Turn number to stop at (0 = initial state before any actions).
    /// This is the number of actions already applied, not the hanab.live 1-based turn label.
    /// To inspect what the engine recommends on hanab.live turn N, pass --turn N-1.
    #[arg(long)]
    turn: usize,

    /// Output path for the scenario JSON (e.g. tests/scenarios/search/my_bug/table_state.json).
    #[arg(long)]
    out: PathBuf,

    /// Human-readable description to embed in `scenario_description`.
    #[arg(long, default_value = "")]
    description: String,
}

fn main() {
    let args = Args::parse();

    let json = fs::read_to_string(&args.replay)
        .unwrap_or_else(|e| panic!("could not read {}: {e}", args.replay.display()));
    let game = Game::from_json(&json)
        .unwrap_or_else(|e| panic!("could not parse {}: {e}", args.replay.display()));

    let conv = HGroupConventionSet::default();
    let mut runner = ReplayRunner::from_hanablive(&game, &conv)
        .unwrap_or_else(|e| panic!("could not build runner: {e}"));
    runner
        .step_to_turn(args.turn)
        .unwrap_or_else(|e| panic!("could not step to turn {}: {e}", args.turn));

    let mut scenario_value = to_scenario_json(&runner.game);
    scenario_value["scenario_description"] = serde_json::Value::String(args.description.clone());

    // Roundtrip check: compare the engine's recommendation before and after serialisation.
    let strategy = TreeActionSelectionStrategy::default();
    let original_action = runner.engine_recommendation(&strategy);

    let json_str =
        serde_json::to_string_pretty(&scenario_value).expect("could not serialise scenario JSON");
    let loaded_scenario = parse_scenario(&json_str);

    // Compare engine recommendations before and after serialisation.
    let loaded_game = knowledge_aware_from_scenario(
        &loaded_scenario,
        eel::game::variant::test_variants::NO_VARIANT,
    );
    let active = loaded_game.table_state.active_player_index;
    let roundtrip_pov = loaded_game.player_pov(active);
    let roundtrip_action = strategy.select_active_player_action(&roundtrip_pov, &conv);
    if roundtrip_action != original_action {
        eprintln!(
            "warn: roundtrip changes engine recommendation!\n  \
             original:  {original_action:?}\n  \
             roundtrip: {roundtrip_action:?}\n  \
             The scenario format cannot faithfully represent convention state accumulated \
             over prior turns. Use a replay regression test instead."
        );
    }

    if let Some(parent) = args.out.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|e| panic!("could not create output directory: {e}"));
    }
    fs::write(&args.out, &json_str)
        .unwrap_or_else(|e| panic!("could not write {}: {e}", args.out.display()));

    let out_display = args.out.display();
    let scenario_name = args
        .out
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|f| f.to_str())
        .unwrap_or("scenario_name");

    eprintln!("Wrote scenario to {out_display}.");
    eprintln!();
    eprintln!(
        "Engine recommendation at turn {}: {original_action:?}",
        args.turn
    );
    eprintln!();
    eprintln!("Add a test in tests/search_regression.rs:");
    eprintln!();
    eprintln!("    #[test]");
    eprintln!("    fn {scenario_name}_picks_correct_action() {{");
    eprintln!("        let action = search_best_action(\"{scenario_name}\");");
    eprintln!("        // TODO: assert!(matches!(action, ...));");
    eprintln!("    }}");
}
