//! Replay a recorded liveplay transcript offline and print the engine's recommendation at a turn.
//!
//! The transcript is the per-game log written by the debug-build `LiveplayRecorder`
//! (`logs/liveplay-*.log`): one raw websocket frame per line. This tool feeds it through the
//! **exact** live `BotState` code path — including hidden own-hand draws and the server's clue
//! `list`/`Turn` ordering — so it reproduces what the live bot actually decided, which the
//! full-deck `replay.json` reconstruction cannot.
//!
//! Usage:
//!   cargo run --bin replay_transcript -- \
//!       --transcript logs/liveplay-20260607T120000.000Z.log \
//!       --turn 44
//!
//! `--turn` is 1-based and matches the `snapshot`/replay-regression harnesses: turn 1 is the
//! opening decision, turn N is the position after N-1 actions have been applied.
use std::fs;
use std::path::PathBuf;

use clap::Parser;

use eel::engine::convention::hgroup::h_group_convention_set::HGroupLevel;
use eel::hanablive::bot::bot_state::BotState;
use eel::hanablive::replay_transcript::replay_to_turn;

#[derive(Parser, Debug)]
#[command(about = "Replay a liveplay transcript and print the engine recommendation at a turn")]
struct Args {
    /// Path to a transcript file recorded by the liveplay recorder (`logs/liveplay-*.log`).
    #[arg(long)]
    transcript: PathBuf,

    /// 1-based turn to stop at (turn 1 = opening decision, before any action is applied).
    #[arg(long)]
    turn: usize,
}

fn main() {
    let args = Args::parse();

    let transcript = fs::read_to_string(&args.transcript)
        .unwrap_or_else(|e| panic!("could not read {}: {e}", args.transcript.display()));

    let replay = replay_to_turn(&transcript, HGroupLevel::One, args.turn).unwrap_or_else(|e| {
        panic!(
            "could not replay {} to turn {}: {e}",
            args.transcript.display(),
            args.turn
        )
    });

    let action = replay.engine_recommendation();

    let BotState::Playing {
        our_player_index,
        game,
        ..
    } = &replay.state
    else {
        unreachable!("replay_to_turn only returns Ok in the Playing state");
    };
    let active = game.table_state.active_player_index;

    println!("turn {} reached", args.turn);
    println!("active player: {active}  (bot is player {our_player_index})");
    println!(
        "{}",
        if active == *our_player_index {
            "it is the bot's turn"
        } else {
            "it is NOT the bot's turn (showing what the active player would do)"
        }
    );
    println!("engine recommendation: {action:?}");
}
