#!/usr/bin/env bash
# Usage: scripts/diag.sh <replay_file.json> <turn> [test_name]
#
# Generates tests/_diag.rs from a template, runs it with --nocapture,
# and leaves the file in place (it is gitignored).
#
# Examples:
#   scripts/diag.sh game_0042.json 22
#   scripts/diag.sh game_0042.json 22 save_b5

set -euo pipefail

REPLAY=${1:?Usage: $0 <replay_file.json> <turn> [test_name]}
TURN=${2:?Usage: $0 <replay_file.json> <turn> [test_name]}
NAME=${3:-debug}

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$SCRIPT_DIR/.."

cat > "$ROOT/tests/_diag.rs" << RUST
use eel::engine::convention::convention_set::ConventionSet;
use eel::engine::convention::hgroup::h_group_convention_set::HGroupConventionSet;
use eel::engine::knowledge::player_pov::PlayerPOV;
use eel::engine::replay::reconstruct::ReplayRunner;
use eel::engine::tree_action_selection_strategy::TreeActionSelectionStrategy;
use eel::external::hanablive::Game;

#[test]
fn diag_${NAME}() {
    let path = format!(
        "{}/tests/replays/${REPLAY}",
        env!("CARGO_MANIFEST_DIR")
    );
    let json = std::fs::read_to_string(&path).unwrap();
    let game = Game::from_json(&json).unwrap();
    let conv = HGroupConventionSet::default();
    let mut runner = ReplayRunner::from_hanablive(&game, &conv).unwrap();
    runner.step_to_turn(${TURN}).unwrap();

    let strategy = TreeActionSelectionStrategy::default();
    println!("recommendation: {:?}", runner.engine_recommendation(&strategy));

    let pov = runner
        .game
        .player_pov(runner.game.table_state.active_player_index);
    println!("active player: {}", pov.active_player_index());
    println!("stacks: {:?}", pov.table_state().playing_stacks);

    let knowledge = pov.team_knowledge().player(pov.active_player_index());
    let mut hand = knowledge.own_hand;
    while hand != 0 {
        let idx = hand.trailing_zeros() as u8;
        let empathy = knowledge.combined_possible_identities(
            idx,
            pov.table_state(),
            &pov.static_data().variant,
        );
        let inferred = knowledge.possible_identities(idx);
        println!(
            "  deck[{}]: empathy={:025b} inferred={:?} touched={} signals={:?}",
            idx,
            empathy.as_bits(),
            inferred.map(|m| format!("{:025b}", m.as_bits())),
            pov.is_touched(idx),
            &knowledge.signals[idx as usize],
        );
        hand &= !(1u64 << idx);
    }
    println!(
        "playable_cards: {:025b}",
        pov.table_state().playable_cards(pov.static_data())
    );

    println!("hypotheses ({}):", knowledge.hypotheses.len());
    for h in &knowledge.hypotheses {
        println!(
            "  id={} tier={} cohort={} alt_group={:?} immediate={:?} trigger={:?}",
            h.id,
            h.tier,
            h.cohort_id,
            h.alt_group,
            h.immediate,
            h.trigger
        );
    }

    for tech in conv.techs() {
        let actions = tech.game_actions(&pov);
        if !actions.is_empty() {
            println!("tech {} proposes: {:?}", tech.name(), actions);
        }
    }
}
RUST

echo "Generated tests/_diag.rs — running diag_${NAME}..."
cd "$ROOT"
cargo test --test _diag "diag_${NAME}" -- --nocapture
