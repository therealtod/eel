//! Builder for constructing a hanab.live-compatible [`Game`] replay during selfplay.
//!
//! # Usage
//! ```rust,ignore
//! let mut builder = GameBuilder::new(player_names, deck_cards);
//! builder.push_play(card_order);
//! builder.push_discard(card_order);
//! builder.push_color_clue(target_player, suit_index);
//! builder.push_rank_clue(target_player, rank);
//! let game = builder.finish();
//! let json = game.to_json_pretty()?;
//! ```

use super::{Action, ActionType, Card, Game, GameOptions};

/// Incrementally builds a hanab.live [`Game`] replay.
pub struct GameBuilder {
    players: Vec<String>,
    deck: Vec<Card>,
    actions: Vec<Action>,
    options: Option<GameOptions>,
    seed: Option<String>,
}

impl GameBuilder {
    /// Create a new builder.
    ///
    /// * `players` – player names (in seat order).
    /// * `deck` – the full ordered deck as `(suit_index, rank)` pairs, oldest card first.
    pub fn new(players: Vec<String>, deck: Vec<Card>) -> Self {
        Self {
            players,
            deck,
            actions: Vec::new(),
            options: None,
            seed: None,
        }
    }

    /// Attach a `GameOptions` (e.g. the variant name).
    pub fn with_options(mut self, options: GameOptions) -> Self {
        self.options = Some(options);
        self
    }

    /// Attach an RNG seed string for hanab.live.
    pub fn with_seed(mut self, seed: impl Into<String>) -> Self {
        self.seed = Some(seed.into());
        self
    }

    // ── action helpers ────────────────────────────────────────────────────────

    /// Record a play action. `card_order` is the 0-based deck position of the card played.
    pub fn push_play(&mut self, card_order: usize) {
        self.actions.push(Action {
            action_type: ActionType::Play,
            target: card_order,
            value: None,
        });
    }

    /// Record a discard action. `card_order` is the 0-based deck position of the card discarded.
    pub fn push_discard(&mut self, card_order: usize) {
        self.actions.push(Action {
            action_type: ActionType::Discard,
            target: card_order,
            value: None,
        });
    }

    /// Record a colour clue. `target_player` is the seat index of the clue recipient;
    /// `suit_index` is 0-based (0=Red, 1=Yellow, 2=Green, 3=Blue, 4=Purple for NO_VARIANT).
    pub fn push_color_clue(&mut self, target_player: usize, suit_index: usize) {
        self.actions.push(Action {
            action_type: ActionType::ColorClue,
            target: target_player,
            value: Some(suit_index),
        });
    }

    /// Record a rank clue. `target_player` is the seat index of the clue recipient;
    /// `rank` is 1-based.
    pub fn push_rank_clue(&mut self, target_player: usize, rank: usize) {
        self.actions.push(Action {
            action_type: ActionType::RankClue,
            target: target_player,
            value: Some(rank),
        });
    }

    /// Consume the builder and produce the final [`Game`].
    pub fn finish(self) -> Game {
        Game {
            players: self.players,
            deck: self.deck,
            actions: self.actions,
            options: self.options,
            notes: None,
            characters: None,
            id: None,
            seed: self.seed,
        }
    }
}
