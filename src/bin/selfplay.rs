/// Self-play runner for evaluating DefaultEvaluator parameter configurations.
///
/// Runs N complete Hanabi games using the H-Group convention set, reports score statistics.
/// Each game uses a fresh shuffled deck; the bot operates in hidden-information mode (each
/// player only knows what their conventions tell them). Game mechanics (stacks, strikes, draws)
/// are tracked with full card-identity knowledge, so scores are exact.
///
/// Usage:
///   cargo run --release --bin selfplay -- --games 200 --players 3
///   cargo run --release --bin selfplay -- --games 200 --seed 42 --verbose
use std::time::SystemTime;

use clap::Parser;
use rand::seq::SliceRandom;
use rand::rngs::SmallRng;
use rand::SeedableRng;

use eel::engine::action_selection_strategy::ActionSelectionStrategy;
use eel::engine::convention::convention_set::ConventionSet;
use eel::engine::convention::hgroup::h_group_convention_set::HGroupConventionSet;
use eel::engine::convention::hgroup::tech::blind_play::BlindPlay;
use eel::engine::convention::hgroup::tech::critical_save::{ColorCriticalSave, RankCriticalSave};
use eel::engine::convention::hgroup::tech::delayed_play_clue::DelayedPlayClue;
use eel::engine::convention::hgroup::tech::direct_play_clue::DirectPlayClue;
use eel::engine::convention::hgroup::tech::discard_chop::DiscardChop;
use eel::engine::convention::hgroup::tech::five_save::FiveSave;
use eel::engine::convention::hgroup::tech::play_known_playable::PlayKnownPlayable;
use eel::engine::convention::hgroup::tech::simple_finesse::SimpleFinesse;
use eel::engine::convention::hgroup::tech::simple_prompt::SimplePrompt;
use eel::engine::convention::hgroup::tech::two_save::TwoSave;
use eel::engine::knowledge::knowledge_update::Hypothesis;
use eel::engine::knowledge_aware_game_state::KnowledgeAwareGameState;
use eel::engine::tree_action_selection_strategy::TreeActionSelectionStrategy;
use eel::game::action::game_action::GameAction;
use eel::game::card::{CardDeckIndex, VariantCardId};
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
}

fn hand_size(players: u8) -> usize {
    if players <= 3 { 5 } else { 4 }
}

fn build_convention_set() -> HGroupConventionSet {
    HGroupConventionSet::new(vec![
        Box::new(PlayKnownPlayable),
        Box::new(BlindPlay),
        Box::new(DirectPlayClue),
        Box::new(DelayedPlayClue),
        Box::new(SimplePrompt),
        Box::new(SimpleFinesse),
        Box::new(ColorCriticalSave),
        Box::new(RankCriticalSave),
        Box::new(FiveSave),
        Box::new(TwoSave),
        Box::new(DiscardChop),
    ])
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

/// Deal the initial hands. Each player gets `hand_size` cards, oldest card drawn first.
/// After dealing, `game.next_deck_index` points to the next undealt card.
fn deal_initial_hands(
    game: &mut KnowledgeAwareGameState,
    actual_deck: &[VariantCardId],
    num_players: usize,
    hand_size: usize,
) {
    for player in 0..num_players {
        // update_with_draw_action (inside update_with_draw_action_of_specific_card) uses
        // player_on_turn_index to add the card to the correct hand.
        game.table_state.active_player_index = player;
        for slot in 0..hand_size {
            let deck_idx = (player * hand_size + slot) as CardDeckIndex;
            let card_id = actual_deck[deck_idx as usize];
            game.update_with_draw_action_of_specific_card(player, deck_idx, card_id);
        }
    }
    game.table_state.active_player_index = 0;
    game.next_deck_index = (num_players * hand_size) as u8;
}

/// Draw the next card from the actual deck for `player`, if the deck is non-empty.
fn draw_next_card(
    game: &mut KnowledgeAwareGameState,
    player: usize,
    actual_deck: &[VariantCardId],
) {
    if game.table_state.deck.current_size == 0 {
        return;
    }
    // player_on_turn_index must equal `player` so the card lands in the right hand.
    debug_assert_eq!(game.table_state.active_player_index, player);
    let deck_idx = game.next_deck_index;
    game.next_deck_index += 1;
    let card_id = actual_deck[deck_idx as usize];
    game.update_with_draw_action_of_specific_card(player, deck_idx, card_id);
}

/// Apply a play action in spectator mode: update stacks/strikes with the actual card identity,
/// propagate convention knowledge to teammates, then draw a replacement card.
fn apply_play_spectator(
    game: &mut KnowledgeAwareGameState,
    card_deck_index: CardDeckIndex,
    actual_deck: &[VariantCardId],
    convention_set: &dyn ConventionSet,
    static_data: &StaticGameData,
    next_hypothesis_id: &mut u32,
) {
    let p = game.table_state.active_player_index;
    let actual_id = actual_deck[card_deck_index as usize];
    let action = GameAction::Play { player_index: p, card_deck_index, turn: game.table_state.current_turn };

    // Capture all matching techs' hypotheses BEFORE mutating state.
    let actor_hypotheses: Vec<Hypothesis> = {
        let pov = game.player_pov(p);
        convention_set
            .techs()
            .iter()
            .filter(|tech| tech.matches_action(&action, &[], &pov))
            .map(|tech| tech.knowledge_updates(&action, &[], &pov))
            .filter(|h| !h.is_empty())
            .collect()
    };

    // Apply game mechanics with the actual card identity (correctly updates playing stacks
    // and strike_tokens, and removes the card from own_hand).
    game.update_with_play_action_of_specific_card(card_deck_index, actual_id);

    // Draw a replacement card with its true identity revealed to all non-drawing players.
    draw_next_card(game, p, actual_deck);

    // Propagate convention knowledge to teammates (e.g. finesse / prompt signals).
    let num_players = static_data.number_of_players as usize;
    let cohort_id = *next_hypothesis_id;
    *next_hypothesis_id += 1;
    for target in (0..num_players).filter(|&t| t != p) {
        let own_hand = game.team_knowledge.player(target).own_hand;
        let filtered: Vec<Hypothesis> = actor_hypotheses
            .iter()
            .map(|h| Hypothesis {
                immediate: h
                    .immediate
                    .iter()
                    .filter(|u| own_hand & (1 << u.card_deck_index()) != 0)
                    .cloned()
                    .collect(),
                trigger: h.trigger.clone(),
            })
            .filter(|h| !h.is_empty())
            .collect();
        game.team_knowledge.player_mut(target).apply_cohort(
            cohort_id,
            filtered,
            next_hypothesis_id,
            &static_data.variant,
        );
    }
}

/// Apply a discard action in spectator mode: record the actual card in the discard pile,
/// return the clue token, then draw a replacement card.
///
/// Discards carry no convention knowledge updates in H-Group, so no propagation step needed.
fn apply_discard_spectator(
    game: &mut KnowledgeAwareGameState,
    card_deck_index: CardDeckIndex,
    actual_deck: &[VariantCardId],
) {
    let p = game.table_state.active_player_index;
    let actual_id = actual_deck[card_deck_index as usize];
    game.update_with_discard_action_of_specific_card(card_deck_index, actual_id);
    draw_next_card(game, p, actual_deck);
}

/// Run one complete game and return the final score (0–25).
fn run_game(
    static_data: &StaticGameData,
    convention_set: &dyn ConventionSet,
    strategy: &TreeActionSelectionStrategy,
    rng: &mut SmallRng,
) -> u8 {
    let actual_deck = shuffled_deck(rng);
    let num_players = static_data.number_of_players as usize;
    let hs = hand_size(static_data.number_of_players);

    let mut game = KnowledgeAwareGameState::new(static_data.clone());
    deal_initial_hands(&mut game, &actual_deck, num_players, hs);

    // None = deck not yet empty; Some(n) = n turns remaining in the final round.
    let mut final_round: Option<usize> = None;
    let mut next_hypothesis_id: u32 = 0;

    loop {
        if game.table_state.is_terminal(static_data) {
            break;
        }

        let current = game.table_state.active_player_index;
        let action = {
            let pov = game.player_pov(current);
            strategy.select_active_player_action(&pov, convention_set)
        };

        match &action {
            GameAction::Play { card_deck_index, .. } => {
                apply_play_spectator(&mut game, *card_deck_index, &actual_deck, convention_set, static_data, &mut next_hypothesis_id);
            }
            GameAction::Discard { card_deck_index, .. } => {
                apply_discard_spectator(&mut game, *card_deck_index, &actual_deck);
            }
            GameAction::Clue { .. } => {
                // Clues don't draw cards, so apply() handles everything correctly.
                game.apply(&action, convention_set);
            }
            GameAction::Draw { .. } => {}
        }

        game.advance_turn();

        // Detect the start of the final round (deck just became empty after a draw).
        if final_round.is_none() && game.table_state.deck.current_size == 0 {
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

    game.table_state.score(&static_data.variant)
}

fn main() {
    let args = Args::parse();

    let seed = args.seed.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    });

    eprintln!("seed: {seed}  players: {}  games: {}", args.players, args.games);

    let mut rng = SmallRng::seed_from_u64(seed);
    let static_data = StaticGameData { number_of_players: args.players, variant: NO_VARIANT };
    let convention_set = build_convention_set();
    let strategy = TreeActionSelectionStrategy::default();

    let mut scores: Vec<u8> = Vec::with_capacity(args.games as usize);
    let progress_every = (args.games / 10).max(1);

    for game_num in 1..=args.games {
        let score = run_game(&static_data, &convention_set, &strategy, &mut rng);
        scores.push(score);

        if args.verbose {
            println!("game {game_num:>4}: {score}");
        } else if game_num % progress_every == 0 {
            eprint!(".");
        }
    }
    if !args.verbose {
        eprintln!();
    }

    // Summary statistics
    let n = scores.len() as f64;
    let mean = scores.iter().map(|&s| s as f64).sum::<f64>() / n;
    let variance = scores
        .iter()
        .map(|&s| { let d = s as f64 - mean; d * d })
        .sum::<f64>()
        / n;
    let std_dev = variance.sqrt();
    let min = *scores.iter().min().unwrap();
    let max = *scores.iter().max().unwrap();

    let perfect = scores.iter().filter(|&&s| s == 25).count();

    println!("\n=== {} games, {}p ===", args.games, args.players);
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
}
