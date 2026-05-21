/// Self-play runner for evaluating DefaultEvaluator parameter configurations.
///
/// Runs N complete Hanabi games using the H-Group convention set, reports score statistics.
/// Each game uses a fresh shuffled deck; the bot operates in hidden-information mode (each
/// player only knows what their conventions tell them). Game mechanics (stacks, strikes, draws)
/// are tracked with full card-identity knowledge, so scores are exact.
///
/// When `--log-failures-below` is set every game whose score falls strictly below the threshold
/// is written as a hanab.live-compatible JSON replay to:
///
///   logs/<score>/game_<N>.json
///
/// (the `logs/` directory tree is created automatically if it does not already exist.)
///
/// The JSON can be pasted into hanab.live → "Watch Specific Replay" to step through the game.
///
/// Usage:
///   cargo run --release --bin selfplay -- --games 200 --players 3
///   cargo run --release --bin selfplay -- --games 200 --seed 42 --verbose
///   cargo run --release --bin selfplay -- --games 200 --log-failures-below 20
use std::fs;
use std::path::PathBuf;
use std::time::{Instant, SystemTime};

use clap::Parser;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;

use eel::engine::action_selection_strategy::ActionSelectionStrategy;
use eel::engine::convention::convention_set::ConventionSet;
use eel::engine::convention::hgroup::h_group_convention_set::HGroupConventionSet;
use eel::engine::replay::reconstruct::{ReplayRunner, variant_card_id_to_hanablive};
use eel::engine::tree_action_selection_strategy::TreeActionSelectionStrategy;
use eel::external::hanablive::{GameBuilder, GameOptions};
use eel::game::action::game_action::GameAction;
use eel::game::card::VariantCardId;
use eel::game::clue_type::ClueType;
use eel::game::static_game_data::StaticGameData;
use eel::game::variant::test_variants::NO_VARIANT;

#[derive(Parser, Debug)]
#[command(about = "Run self-play games to benchmark evaluator parameters")]
struct Args {
    /// Number of complete games to simulate.
    #[arg(long, default_value_t = 200)]
    games: u32,

    /// Number of players per game (2–5).
    #[arg(long, default_value_t = 3)]
    players: u8,

    /// RNG seed for reproducibility. Omit for a random seed.
    #[arg(long)]
    seed: Option<u64>,

    /// Print each game's score as it completes.
    #[arg(long, default_value_t = false)]
    verbose: bool,

    /// Write a hanab.live replay JSON for every game that scores strictly below this value.
    /// Files are written to logs/<score>/game_<N>.json.
    #[arg(long)]
    log_failures_below: Option<u8>,
}

/// Produce a shuffled list of all card IDs for NO_VARIANT (50 cards).
fn shuffled_deck(rng: &mut SmallRng) -> Vec<VariantCardId> {
    let mut deck: Vec<VariantCardId> = NO_VARIANT
        .card_copies_count_by_id
        .iter()
        .enumerate()
        .flat_map(|(id, &copies)| std::iter::repeat(id as VariantCardId).take(copies as usize))
        .collect();
    deck.shuffle(rng);
    deck
}

/// Run one complete game and return the final score (0–25).
///
/// When `log_failures_below` is `Some(t)` and the final score < `t`, the game is serialised
/// to `logs/<score>/game_<game_num>.json` in hanab.live replay format.
fn run_game(
    static_data: &StaticGameData,
    convention_set: &dyn ConventionSet,
    strategy: &TreeActionSelectionStrategy,
    rng: &mut SmallRng,
    log_failures_below: Option<u8>,
    game_num: u32,
) -> u8 {
    let actual_deck = shuffled_deck(rng);
    let num_players = static_data.number_of_players as usize;

    // Build the hanab.live deck (same order as actual_deck).
    let hanablive_deck = actual_deck
        .iter()
        .map(|&id| variant_card_id_to_hanablive(id, static_data))
        .collect();

    let player_names: Vec<String> = (0..num_players).map(|i| format!("Bot{i}")).collect();

    let logging_enabled = log_failures_below.is_some();
    let mut builder = if logging_enabled {
        Some(
            GameBuilder::new(player_names, hanablive_deck).with_options({
                let mut opts = GameOptions::default();
                opts.variant = Some("No Variant".to_string());
                opts
            }),
        )
    } else {
        None
    };

    let mut runner = ReplayRunner::from_deck(actual_deck, static_data.clone(), convention_set);

    // None = deck not yet empty; Some(n) = n turns remaining in the final round.
    let mut final_round: Option<usize> = None;

    loop {
        if runner.game.table_state.is_terminal(static_data) {
            break;
        }

        let current = runner.game.table_state.active_player_index;
        let action = {
            let pov = runner.game.player_pov(current);
            strategy.select_active_player_action(&pov, convention_set)
        };

        // Record the action in hanab.live format before we mutate state.
        if let Some(ref mut b) = builder {
            match &action {
                GameAction::Play {
                    card_deck_index, ..
                } => {
                    b.push_play(*card_deck_index as usize);
                }
                GameAction::Discard {
                    card_deck_index, ..
                } => {
                    b.push_discard(*card_deck_index as usize);
                }
                GameAction::Clue {
                    clue, player_index, ..
                } => match clue.clue_type {
                    ClueType::Color => b.push_color_clue(*player_index, clue.clue_value as usize),
                    ClueType::Rank => b.push_rank_clue(*player_index, clue.clue_value as usize),
                },
                GameAction::Draw { .. } => {}
            }
        }

        runner.apply_strategy_action(&action);
        runner.game.advance_turn();

        // Detect the start of the final round (deck just became empty after a draw).
        if final_round.is_none() && runner.game.table_state.deck.current_size == 0 {
            // The player who just went already took their last turn; num_players-1 remain.
            final_round = Some(num_players - 1);
        }

        // Decrement the final-round counter after every turn once active.
        if let Some(n) = final_round.as_mut() {
            if *n > 0 {
                *n -= 1;
            } else {
                break; // all final-round turns consumed
            }
        }
    }

    let score = runner.game.table_state.score(&static_data.variant);

    if let Some(threshold) = log_failures_below {
        if score < threshold {
            if let Some(b) = builder {
                let replay = b.finish();
                let dir = PathBuf::from("logs").join(score.to_string());
                if let Err(e) = fs::create_dir_all(&dir) {
                    eprintln!("warn: could not create log directory {dir:?}: {e}");
                } else {
                    let path = dir.join(format!("game_{game_num}.json"));
                    match replay.to_json_pretty() {
                        Ok(json) => {
                            if let Err(e) = fs::write(&path, &json) {
                                eprintln!("warn: could not write replay to {path:?}: {e}");
                            }
                        }
                        Err(e) => eprintln!("warn: could not serialise replay: {e}"),
                    }
                }
            }
        }
    }

    score
}

fn main() {
    let args = Args::parse();

    let seed = args.seed.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    });

    eprintln!(
        "seed: {seed}  players: {}  games: {}",
        args.players, args.games
    );

    let mut rng = SmallRng::seed_from_u64(seed);
    let static_data = StaticGameData {
        number_of_players: args.players,
        variant: NO_VARIANT,
    };
    let convention_set = HGroupConventionSet::default();
    let strategy = TreeActionSelectionStrategy::default();

    let mut scores: Vec<u8> = Vec::with_capacity(args.games as usize);
    let progress_every = (args.games / 10).max(1);

    let start = Instant::now();

    for game_num in 1..=args.games {
        let score = run_game(
            &static_data,
            &convention_set,
            &strategy,
            &mut rng,
            args.log_failures_below,
            game_num,
        );
        scores.push(score);

        if args.verbose {
            println!("game {game_num:>4}: {score}");
        } else if game_num % progress_every == 0 {
            eprintln!("currently simulating game {game_num}");
        }
    }

    let elapsed = start.elapsed();

    // Summary statistics
    let n = scores.len() as f64;
    let mean = scores.iter().map(|&s| s as f64).sum::<f64>() / n;
    let variance = scores
        .iter()
        .map(|&s| {
            let d = s as f64 - mean;
            d * d
        })
        .sum::<f64>()
        / n;
    let std_dev = variance.sqrt();
    let min = *scores.iter().min().unwrap();
    let max = *scores.iter().max().unwrap();

    let perfect = scores.iter().filter(|&&s| s == 25).count();

    println!("\n=== {} games, {}p ===", args.games, args.players);
    println!("time:    {:.2}s", elapsed.as_secs_f64());
    println!("mean:    {mean:.3}");
    println!("std dev: {std_dev:.3}");
    println!("min/max: {min} / {max}");
    println!("perfect: {perfect} ({:.1}%)", perfect as f64 / n * 100.0);

    // Score distribution histogram
    println!("\nScore distribution:");
    let mut dist = [0u32; 26];
    for &s in &scores {
        dist[s as usize] += 1;
    }
    for (score, &count) in dist.iter().enumerate().filter(|&(_, &c)| c > 0) {
        let pct = count as f64 / n * 100.0;
        let bar = "#".repeat((pct / 2.0).round() as usize);
        println!("{score:>2}: {bar:<50} {count:>4} ({pct:4.1}%)");
    }

    if args.log_failures_below.is_some() {
        eprintln!("\nReplays written to logs/<score>/game_<N>.json");
    }
}
