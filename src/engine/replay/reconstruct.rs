use std::fmt;

use smallvec::SmallVec;

use crate::engine::action_selection_strategy::ActionSelectionStrategy;
use crate::engine::convention::convention_set::ConventionSet;
use crate::engine::knowledge::knowledge_update::{Hypothesis, HypothesisId};
use crate::engine::knowledge_aware_game_state::{collect_hypotheses, KnowledgeAwareGameState};
use crate::engine::tree_action_selection_strategy::TreeActionSelectionStrategy;
use crate::external::hanablive::{Action, ActionType, Card};
use crate::game::action::game_action::GameAction;
use crate::game::card::{CardDeckIndex, VariantCardId};
use crate::game::clue::Clue;
use crate::game::clue_type::ClueType;
use crate::game::static_game_data::StaticGameData;
use crate::game::variant::test_variants::NO_VARIANT;
use crate::game::MAX_HAND_SIZE;

#[derive(Debug)]
pub enum ReplayError {
    UnsupportedVariant(Option<String>),
    PastEnd,
    BackwardsStep { from: usize, to: usize },
    MalformedClue(String),
    InvalidTarget { target: usize, deck_size: usize },
}

impl fmt::Display for ReplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReplayError::UnsupportedVariant(v) => write!(f, "unsupported variant: {v:?}"),
            ReplayError::PastEnd => write!(f, "cursor past end of replay"),
            ReplayError::BackwardsStep { from, to } => {
                write!(f, "cannot step backwards from turn {from} to turn {to}")
            }
            ReplayError::MalformedClue(s) => write!(f, "malformed clue action: {s}"),
            ReplayError::InvalidTarget { target, deck_size } => {
                write!(f, "invalid action target {target} (deck size {deck_size})")
            }
        }
    }
}

impl std::error::Error for ReplayError {}

#[derive(Debug)]
pub enum AppliedAction {
    Play,
    Discard,
    Clue,
}

pub struct ReplayRunner<'a> {
    pub game: KnowledgeAwareGameState,
    pub actual_deck: Vec<VariantCardId>,
    pub static_data: StaticGameData,
    convention_set: &'a dyn ConventionSet,
    /// Monotonic counter for hypothesis cohort IDs. Separate from the one
    /// inside KnowledgeAwareGameState so spectator-mode paths don't collide
    /// with search-mode paths.
    next_hypothesis_id: HypothesisId,
    actions: Vec<Action>,
    cursor: usize,
}

impl<'a> ReplayRunner<'a> {
    /// Build a runner from a parsed hanab.live `Game`.
    ///
    /// Asserts that `options.variant == "No Variant"` and refuses anything else.
    /// Deals the initial hands; the runner is positioned before the first action
    /// (`current_turn() == 0`).
    pub fn from_hanablive(
        game: &crate::external::hanablive::Game,
        convention_set: &'a dyn ConventionSet,
    ) -> Result<Self, ReplayError> {
        let variant_name = game
            .options
            .as_ref()
            .and_then(|o| o.variant.as_deref());
        if variant_name != Some("No Variant") {
            return Err(ReplayError::UnsupportedVariant(
                variant_name.map(|s| s.to_string()),
            ));
        }

        let num_players = game.players.len() as u8;
        let static_data = StaticGameData {
            number_of_players: num_players,
            variant: NO_VARIANT,
        };

        let stacks_size = NO_VARIANT.stacks_size as usize;
        let actual_deck: Vec<VariantCardId> = game
            .deck
            .iter()
            .map(|card| card.suit_index * stacks_size + (card.rank as usize - 1))
            .collect();

        // Filter EndGame sentinels — they carry no state and would confuse the cursor.
        let actions: Vec<Action> = game
            .actions
            .iter()
            .filter(|a| a.action_type != ActionType::EndGame)
            .cloned()
            .collect();

        let mut runner = ReplayRunner {
            game: KnowledgeAwareGameState::new(static_data.clone()),
            actual_deck,
            static_data,
            convention_set,
            next_hypothesis_id: 0,
            actions,
            cursor: 0,
        };

        let hs = hand_size_for(num_players);
        runner.deal_initial_hands(num_players as usize, hs);

        Ok(runner)
    }

    /// Build a runner from a pre-shuffled deck (for selfplay).
    ///
    /// Deals initial hands; `actions` starts empty. Call `apply_strategy_action`
    /// to advance state one action at a time, then `game.advance_turn()` to move
    /// to the next player.
    pub fn from_deck(
        actual_deck: Vec<VariantCardId>,
        static_data: StaticGameData,
        convention_set: &'a dyn ConventionSet,
    ) -> Self {
        let num_players = static_data.number_of_players;
        let hs = hand_size_for(num_players);
        let mut runner = ReplayRunner {
            game: KnowledgeAwareGameState::new(static_data.clone()),
            actual_deck,
            static_data,
            convention_set,
            next_hypothesis_id: 0,
            actions: Vec::new(),
            cursor: 0,
        };
        runner.deal_initial_hands(num_players as usize, hs);
        runner
    }

    /// Apply the next recorded action. Errors if the cursor is past the end or the
    /// action is malformed.
    pub fn step(&mut self) -> Result<AppliedAction, ReplayError> {
        if self.cursor >= self.actions.len() {
            return Err(ReplayError::PastEnd);
        }

        let action = self.actions[self.cursor].clone();

        let result = match action.action_type {
            ActionType::Play => {
                let deck_idx = action.target as CardDeckIndex;
                if action.target >= self.actual_deck.len() {
                    return Err(ReplayError::InvalidTarget {
                        target: action.target,
                        deck_size: self.actual_deck.len(),
                    });
                }
                self.apply_play(deck_idx);
                AppliedAction::Play
            }
            ActionType::Discard => {
                let deck_idx = action.target as CardDeckIndex;
                if action.target >= self.actual_deck.len() {
                    return Err(ReplayError::InvalidTarget {
                        target: action.target,
                        deck_size: self.actual_deck.len(),
                    });
                }
                self.apply_discard(deck_idx);
                AppliedAction::Discard
            }
            ActionType::ColorClue | ActionType::RankClue => {
                let receiver = action.target;
                let value = action.value.ok_or_else(|| {
                    ReplayError::MalformedClue("clue action missing value".to_string())
                })?;
                let (clue_type, clue_value) = match action.action_type {
                    ActionType::ColorClue => (ClueType::Color, value as u8),
                    ActionType::RankClue => (ClueType::Rank, value as u8),
                    _ => unreachable!(),
                };
                let clue = Clue { clue_type, clue_value };
                let touched = self.compute_touched_cards(receiver, &clue)?;
                let turn = self.game.table_state.current_turn;
                let game_action = GameAction::Clue {
                    player_index: receiver,
                    touched_card_deck_indexes: touched,
                    clue,
                    turn,
                };
                self.game.apply(&game_action, self.convention_set);
                AppliedAction::Clue
            }
            ActionType::EndGame => unreachable!("EndGame filtered in from_hanablive"),
        };

        self.game.advance_turn();
        self.cursor += 1;

        Ok(result)
    }

    /// Step until `current_turn() == target_turn`.
    ///
    /// Errors if `target_turn` exceeds the replay length or is less than
    /// `current_turn()` (backwards seek is not supported; runners are forward-only).
    pub fn step_to_turn(&mut self, target_turn: usize) -> Result<(), ReplayError> {
        if target_turn > self.total_turns() {
            return Err(ReplayError::PastEnd);
        }
        if target_turn < self.cursor {
            return Err(ReplayError::BackwardsStep { from: self.cursor, to: target_turn });
        }
        while self.cursor < target_turn {
            self.step()?;
        }
        Ok(())
    }

    /// Total turns recorded (number of filtered, non-EndGame actions).
    pub fn total_turns(&self) -> usize {
        self.actions.len()
    }

    /// Current turn number (equals the number of actions applied so far).
    pub fn current_turn(&self) -> usize {
        self.cursor
    }

    /// Ask the engine what it would play at the current position.
    pub fn engine_recommendation(&self, strategy: &TreeActionSelectionStrategy) -> GameAction {
        let active = self.game.table_state.active_player_index;
        let pov = self.game.player_pov(active);
        strategy.select_active_player_action(&pov, self.convention_set)
    }

    /// The action the replay says was taken next (before it has been applied).
    /// Returns `None` if the cursor is past the end.
    pub fn next_recorded_action(&self) -> Option<&Action> {
        self.actions.get(self.cursor)
    }

    /// Convert the next recorded action to a `GameAction` using the current
    /// hand state to resolve touched cards for clues.
    /// Returns `None` if the cursor is past the end or the action is malformed.
    pub fn next_recorded_as_game_action(&self) -> Option<GameAction> {
        let action = self.actions.get(self.cursor)?;
        let turn = self.game.table_state.current_turn;
        match action.action_type {
            ActionType::Play => Some(GameAction::Play {
                player_index: self.game.table_state.active_player_index,
                card_deck_index: action.target as CardDeckIndex,
                turn,
            }),
            ActionType::Discard => Some(GameAction::Discard {
                player_index: self.game.table_state.active_player_index,
                card_deck_index: action.target as CardDeckIndex,
                turn,
            }),
            ActionType::ColorClue | ActionType::RankClue => {
                let receiver = action.target;
                let value = action.value?;
                let (clue_type, clue_value) = match action.action_type {
                    ActionType::ColorClue => (ClueType::Color, value as u8),
                    ActionType::RankClue => (ClueType::Rank, value as u8),
                    _ => unreachable!(),
                };
                let clue = Clue { clue_type, clue_value };
                let touched = self.compute_touched_cards(receiver, &clue).ok()?;
                Some(GameAction::Clue {
                    player_index: receiver,
                    touched_card_deck_indexes: touched,
                    clue,
                    turn,
                })
            }
            ActionType::EndGame => None,
        }
    }

    /// Apply a strategy-chosen action in selfplay mode.
    ///
    /// Does NOT advance the turn; call `game.advance_turn()` separately so
    /// that selfplay retains control of final-round counting and logging.
    pub fn apply_strategy_action(&mut self, action: &GameAction) {
        match action {
            GameAction::Play { card_deck_index, .. } => self.apply_play(*card_deck_index),
            GameAction::Discard { card_deck_index, .. } => self.apply_discard(*card_deck_index),
            GameAction::Clue { .. } => self.game.apply(action, self.convention_set),
            GameAction::Draw { .. } => {}
        }
    }

    // ── private helpers ───────────────────────────────────────────────────────

    fn deal_initial_hands(&mut self, num_players: usize, hand_size: usize) {
        for player in 0..num_players {
            self.game.table_state.active_player_index = player;
            for slot in 0..hand_size {
                let deck_idx = (player * hand_size + slot) as CardDeckIndex;
                let card_id = self.actual_deck[deck_idx as usize];
                self.game
                    .update_with_draw_action_of_specific_card(player, deck_idx, card_id);
            }
        }
        self.game.table_state.active_player_index = 0;
        self.game.next_deck_index = (num_players * hand_size) as u8;
    }

    fn draw_next_card(&mut self, player: usize) {
        if self.game.table_state.deck.current_size == 0 {
            return;
        }
        debug_assert_eq!(self.game.table_state.active_player_index, player);
        let deck_idx = self.game.next_deck_index;
        self.game.next_deck_index += 1;
        let card_id = self.actual_deck[deck_idx as usize];
        self.game
            .update_with_draw_action_of_specific_card(player, deck_idx, card_id);
    }

    fn apply_play(&mut self, card_deck_index: CardDeckIndex) {
        let p = self.game.table_state.active_player_index;
        let actual_id = self.actual_deck[card_deck_index as usize];
        let action = GameAction::Play {
            player_index: p,
            card_deck_index,
            turn: self.game.table_state.current_turn,
        };

        let actor_hypotheses: Vec<(u8, Hypothesis)> = {
            let pov = self.game.player_pov(p);
            collect_hypotheses(self.convention_set.techs(), &action, &[], &pov)
        };

        self.game
            .update_with_play_action_of_specific_card(card_deck_index, actual_id);
        self.draw_next_card(p);

        let num_players = self.static_data.number_of_players as usize;
        let cohort_id = self.next_hypothesis_id;
        self.next_hypothesis_id += 1;
        for target in (0..num_players).filter(|&t| t != p) {
            let own_hand = self.game.team_knowledge.player(target).own_hand;
            let filtered: Vec<(u8, Hypothesis)> = actor_hypotheses
                .iter()
                .map(|(tier, h)| {
                    (
                        *tier,
                        Hypothesis {
                            immediate: h
                                .immediate
                                .iter()
                                .filter(|u| own_hand & (1 << u.card_deck_index()) != 0)
                                .cloned()
                                .collect(),
                            trigger: h.trigger.clone(),
                        },
                    )
                })
                .filter(|(_, h)| !h.is_empty())
                .collect();
            self.game.team_knowledge.player_mut(target).apply_cohort(
                cohort_id,
                filtered,
                &mut self.next_hypothesis_id,
                &self.static_data.variant,
            );
        }
    }

    fn apply_discard(&mut self, card_deck_index: CardDeckIndex) {
        let p = self.game.table_state.active_player_index;
        let actual_id = self.actual_deck[card_deck_index as usize];
        self.game
            .update_with_discard_action_of_specific_card(card_deck_index, actual_id);
        self.draw_next_card(p);
    }

    fn compute_touched_cards(
        &self,
        receiver: usize,
        clue: &Clue,
    ) -> Result<SmallVec<[CardDeckIndex; MAX_HAND_SIZE]>, ReplayError> {
        let clue_mask = self.static_data.variant.empathy_for_clue(clue).as_bits();
        let hand = &self.game.table_state.hands[receiver];
        let touched: SmallVec<[CardDeckIndex; MAX_HAND_SIZE]> = hand
            .cards()
            .iter()
            .filter(|&&deck_idx| {
                let card_id = self.actual_deck[deck_idx as usize];
                clue_mask & (1 << card_id) != 0
            })
            .copied()
            .collect();
        if touched.is_empty() {
            return Err(ReplayError::MalformedClue(format!(
                "clue {clue:?} touches no cards in player {receiver}'s hand"
            )));
        }
        Ok(touched)
    }
}

/// Return the hand size for a given player count (5 for ≤3 players, 4 otherwise).
pub fn hand_size_for(num_players: u8) -> usize {
    if num_players <= 3 { 5 } else { 4 }
}

/// Convert an internal `VariantCardId` to the hanab.live `Card` representation.
pub fn variant_card_id_to_hanablive(id: VariantCardId, static_data: &StaticGameData) -> Card {
    let suit_index = id / static_data.variant.stacks_size as usize;
    let rank = static_data.variant.rank_of(id);
    Card { suit_index, rank }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::convention::hgroup::h_group_convention_set::HGroupConventionSet;
    use crate::engine::convention::hgroup::tech::blind_play::BlindPlay;
    use crate::engine::convention::hgroup::tech::critical_save::{
        ColorCriticalSave, RankCriticalSave,
    };
    use crate::engine::convention::hgroup::tech::delayed_play_clue::DelayedPlayClue;
    use crate::engine::convention::hgroup::tech::direct_play_clue::DirectPlayClue;
    use crate::engine::convention::hgroup::tech::discard_chop::DiscardChop;
    use crate::engine::convention::hgroup::tech::five_save::FiveSave;
    use crate::engine::convention::hgroup::tech::play_known_playable::PlayKnownPlayable;
    use crate::engine::convention::hgroup::tech::simple_finesse::SimpleFinesse;
    use crate::engine::convention::hgroup::tech::simple_prompt::SimplePrompt;
    use crate::engine::convention::hgroup::tech::two_save::TwoSave;
    use crate::external::hanablive::{Game, GameBuilder, GameOptions};

    fn full_convention_set() -> HGroupConventionSet {
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

    /// Build a minimal 3-player hanab.live Game from a deck + action list.
    fn make_game(
        deck: Vec<(usize, u8)>,
        actions: Vec<(ActionType, usize, Option<usize>)>,
    ) -> Game {
        let cards: Vec<Card> = deck
            .iter()
            .map(|&(suit_index, rank)| Card { suit_index, rank })
            .collect();
        let mut builder = GameBuilder::new(
            vec!["Alice".to_string(), "Bob".to_string(), "Charlie".to_string()],
            cards,
        )
        .with_options({
            let mut opts = GameOptions::default();
            opts.variant = Some("No Variant".to_string());
            opts
        });
        for (action_type, target, value) in actions {
            match action_type {
                ActionType::Play => builder.push_play(target),
                ActionType::Discard => builder.push_discard(target),
                ActionType::ColorClue => {
                    builder.push_color_clue(target, value.unwrap())
                }
                ActionType::RankClue => builder.push_rank_clue(target, value.unwrap()),
                ActionType::EndGame => {}
            }
        }
        builder.finish()
    }

    /// Minimal ordered NO_VARIANT deck (all cards in canonical order): R1 R1 R1 … P5.
    fn ordered_deck() -> Vec<(usize, u8)> {
        let mut deck = Vec::new();
        for suit in 0..5usize {
            for rank in 1u8..=5 {
                let copies: u8 = match rank {
                    1 => 3,
                    2 | 3 | 4 => 2,
                    5 => 1,
                    _ => unreachable!(),
                };
                for _ in 0..copies {
                    deck.push((suit, rank));
                }
            }
        }
        deck
    }

    #[test]
    fn hand_size_for_returns_five_for_three_players() {
        assert_eq!(hand_size_for(3), 5);
    }

    #[test]
    fn hand_size_for_returns_four_for_four_players() {
        assert_eq!(hand_size_for(4), 4);
    }

    #[test]
    fn variant_card_id_to_hanablive_roundtrip() {
        let static_data = StaticGameData {
            number_of_players: 3,
            variant: NO_VARIANT,
        };
        // g1 = suit 2 rank 1 = id 10
        let card = variant_card_id_to_hanablive(10, &static_data);
        assert_eq!(card.suit_index, 2);
        assert_eq!(card.rank, 1);
    }

    #[test]
    fn from_hanablive_rejects_unsupported_variant() {
        let deck = ordered_deck()
            .into_iter()
            .map(|(s, r)| Card { suit_index: s, rank: r })
            .collect();
        let mut game = GameBuilder::new(
            vec!["Alice".to_string(), "Bob".to_string(), "Charlie".to_string()],
            deck,
        )
        .finish();
        game.options = Some({
            let mut o = GameOptions::default();
            o.variant = Some("6 Suits".to_string());
            o
        });
        let conv = full_convention_set();
        assert!(matches!(
            ReplayRunner::from_hanablive(&game, &conv),
            Err(ReplayError::UnsupportedVariant(_))
        ));
    }

    #[test]
    fn regenerates_known_state_after_play_and_discard() {
        // Deck: 15 cards dealt, first few plays/discards known.
        // We apply one play (deck[0]) and one discard (deck[5]) and verify score.
        // Deck[0] = (suit=0, rank=1) = r1 = card_id 0 → playable on empty stack.
        // Deck[5] = (suit=1, rank=1) = y1 = card_id 5 → discard.
        let deck = ordered_deck();
        let game = make_game(
            deck.clone(),
            vec![
                (ActionType::Play, 0, None),   // Alice plays deck[0] = R1
                (ActionType::Discard, 5, None), // Bob discards deck[5] = Y1
            ],
        );
        let conv = full_convention_set();
        let mut runner = ReplayRunner::from_hanablive(&game, &conv).unwrap();

        assert_eq!(runner.current_turn(), 0);
        runner.step().unwrap(); // apply play
        assert_eq!(runner.current_turn(), 1);
        runner.step().unwrap(); // apply discard
        assert_eq!(runner.current_turn(), 2);

        // R1 was played → red stack has 1 card.
        let stacks = &runner.game.table_state.playing_stacks;
        assert_eq!(stacks.stack_size(0), 1);
    }

    #[test]
    fn step_to_turn_is_deterministic_across_runners() {
        let deck = ordered_deck();
        let game = make_game(
            deck.clone(),
            vec![
                (ActionType::Play, 0, None),
                (ActionType::Discard, 5, None),
                (ActionType::RankClue, 2, Some(1)), // Clue rank-1 to Charlie
            ],
        );
        let conv = full_convention_set();

        let mut runner_a = ReplayRunner::from_hanablive(&game, &conv).unwrap();
        let mut runner_b = ReplayRunner::from_hanablive(&game, &conv).unwrap();

        runner_a.step_to_turn(2).unwrap();
        runner_b.step_to_turn(2).unwrap();

        // Both runners should have the same table state at turn 2.
        assert_eq!(
            runner_a.game.table_state.playing_stacks,
            runner_b.game.table_state.playing_stacks
        );
        assert_eq!(
            runner_a.game.table_state.current_turn,
            runner_b.game.table_state.current_turn
        );
        assert_eq!(runner_a.current_turn(), 2);
        assert_eq!(runner_b.current_turn(), 2);
    }

    #[test]
    fn from_deck_and_from_hanablive_agree_on_score() {
        // Apply the same two play actions through both constructors and verify the
        // resulting table state is identical.
        //
        // Deck layout (3 players, hand_size=5):
        //   Alice (player 0): deck[0..4]  = R1 R1 R1 R2 R2 (oldest..newest)
        //   Bob   (player 1): deck[5..9]  = Y1 Y1 Y2 Y2 Y3
        //   Charlie          : deck[10..14] = G1 G1 G2 G2 G3
        //
        // Turn 1 – Alice plays deck[0] (R1): red stack advances to 1.
        // Turn 2 – Bob   plays deck[5] (Y1): yellow stack advances to 1.
        let deck = ordered_deck();
        let game = make_game(
            deck.clone(),
            vec![
                (ActionType::Play, 0, None), // Alice plays deck[0] = R1
                (ActionType::Play, 5, None), // Bob plays deck[5] = Y1
            ],
        );
        let conv = full_convention_set();

        // from_hanablive path
        let mut runner_hl = ReplayRunner::from_hanablive(&game, &conv).unwrap();
        runner_hl.step_to_turn(2).unwrap();

        // from_deck + apply_strategy_action path
        let actual_deck_ids: Vec<VariantCardId> = deck
            .iter()
            .map(|&(suit, rank)| suit * 5 + (rank as usize - 1))
            .collect();
        let static_data = StaticGameData {
            number_of_players: 3,
            variant: NO_VARIANT,
        };
        let mut runner_sd = ReplayRunner::from_deck(actual_deck_ids, static_data, &conv);

        let play0 = GameAction::Play {
            player_index: 0,
            card_deck_index: 0,
            turn: 0,
        };
        let play1 = GameAction::Play {
            player_index: 1,
            card_deck_index: 5,
            turn: 0,
        };
        runner_sd.apply_strategy_action(&play0);
        runner_sd.game.advance_turn();
        runner_sd.apply_strategy_action(&play1);
        runner_sd.game.advance_turn();

        // Both paths should produce identical playing stacks and strike counts.
        assert_eq!(
            runner_hl.game.table_state.playing_stacks,
            runner_sd.game.table_state.playing_stacks
        );
        assert_eq!(
            runner_hl.game.table_state.strike_tokens,
            runner_sd.game.table_state.strike_tokens
        );
    }
}
