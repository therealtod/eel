use crate::game::MAX_HAND_SIZE;
use crate::game::MAX_PLAYERS_IN_GAME;
use crate::game::card::DeckCardsBitField;
use crate::game::card::copies_counting_card_collection::CopiesCountingCardCollection;
use crate::game::card::{CardDeckIndex, VariantCardId, VariantCardsBitField};
use crate::game::clue::Clue;
use crate::game::clue_token_bank::ClueTokenBank;
use crate::game::deck::Deck;
use crate::game::hand::Hand;
use crate::game::playing_stacks::PlayingStacks;
use crate::game::static_game_data::StaticGameData;
use crate::game::variant::Variant;
use smallvec::SmallVec;
use crate::game::state::PlayerIndex;

/// The state of the table of a game of Hanabi
///
/// It captures a snapshot of the state of the table and keeps of the information on the
/// various game entities: playing stacks, discard pile, etc...
///
/// `StaticGameData` is intentionally *not* stored here: it never changes over a game, so
/// cloning it into every alpha-beta node would be wasteful.  Instead, callers hold a single
/// `StaticGameData` and pass `&StaticGameData` to the methods that require it.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TableState {
    pub clue_token_bank: ClueTokenBank,
    pub clue_touched_cards: DeckCardsBitField,
    pub clues_given: u8,
    pub deck: Deck,
    pub hands: [Hand; MAX_PLAYERS_IN_GAME],
    pub all_hand_bits: DeckCardsBitField,
    pub active_player_index: PlayerIndex,
    pub current_turn: usize,
    pub playing_stacks: PlayingStacks,
    pub strike_tokens: u8,
    pub discard_pile: CopiesCountingCardCollection,
}

impl TableState {
    /// Construct a `TableState` from all its constituent parts.
    ///
    /// Intended for use in tests and scenario loading.
    pub fn from_parts(
        clue_token_bank: ClueTokenBank,
        deck: Deck,
        hands: [Hand; MAX_PLAYERS_IN_GAME],
        active_player_index: usize,
        current_turn: usize,
        playing_stacks: PlayingStacks,
        strike_tokens: u8,
        discard_pile: CopiesCountingCardCollection,
    ) -> Self {
        TableState {
            clue_token_bank,
            clue_touched_cards: 0,
            clues_given: 0,
            deck,
            all_hand_bits: 0,
            hands,
            active_player_index,
            current_turn,
            playing_stacks,
            strike_tokens,
            discard_pile,
        }
    }

    pub fn new(static_game_data: &StaticGameData) -> Self {
        Self::from_parts(
            ClueTokenBank::default(),
            Deck::new(&static_game_data.variant),
            Hand::empty_array(),
            0,
            0,
            PlayingStacks::empty(),
            0,
            CopiesCountingCardCollection::empty(),
        )
    }

    /// Returns the index of the player whose turn it is.
    #[inline]
    pub fn active_player_index(&self) -> usize {
        self.active_player_index
    }

    /// Advances to the next player's turn (increments `turn_counter` and wraps player index).
    #[inline]
    pub fn advance_turn(&mut self, num_players: usize) {
        self.current_turn += 1;
        self.active_player_index = (self.active_player_index + 1) % num_players;
    }

    /// Update this [crate::game::game_state::GameState] when the player draws the given card with
    /// the given `player index`
    pub fn update_with_draw_action(&mut self, card_deck_index: CardDeckIndex) {
        self.hands[self.active_player_index].add_card_to_slot_1(card_deck_index);
        self.deck.decrement_size(1);
        self.all_hand_bits |= 1u64 << card_deck_index;
    }

    /// Update this [crate::game::game_state::GameState] when the player with the given
    /// `player_index` tries to play a card
    pub fn update_with_play_action(&mut self, card_deck_index: CardDeckIndex) {
        self.hands[self.active_player_index].remove_card(card_deck_index);
        self.all_hand_bits &= !(1u64 << card_deck_index);
    }

    /// Update this [crate::game::game_state::GameState] when the player with the given
    /// `player_index` tries to play a card whose identity is `card_id`
    pub fn update_with_play_action_of_specific_card(
        &mut self,
        card_deck_index: CardDeckIndex,
        card_id: VariantCardId,
        static_game_data: &StaticGameData,
    ) {
        self.update_with_play_action(card_deck_index);
        self.deck.reveal_card(card_deck_index, card_id);
        let playable_cards = self.playable_cards(static_game_data);
        let is_valid_play = playable_cards & 1 << card_id != 0;
        if is_valid_play {
            let new_stack_size = self
                .playing_stacks
                .add_card(card_id, &static_game_data.variant);
            let bonus_clue_tokens = if new_stack_size == static_game_data.variant.stacks_size {
                static_game_data
                    .variant
                    .bonus_half_clue_tokens_for_completing_stack
            } else {
                0
            };
            self.clue_token_bank.add_tokens(bonus_clue_tokens);
        } else {
            self.strike_tokens += 1;
            self.discard_pile.add_card_with_id(card_id);
        }
    }

    /// Update this [crate::game::game_state::GameState] when the player with the given
    /// `player_index` discards a card
    pub fn update_with_discard_action(
        &mut self,
        card_deck_index: CardDeckIndex,
        static_game_data: &StaticGameData,
    ) {
        self.hands[self.active_player_index].remove_card(card_deck_index);
        self.all_hand_bits &= !(1u64 << card_deck_index);
        self.discard_pile.add_card();
        let bonus_tokens = static_game_data.variant.bonus_half_clue_tokens_for_discard;
        self.clue_token_bank.add_tokens(bonus_tokens);
    }

    /// Update this [crate::game::game_state::GameState] when the player with the given
    /// `player_index` discards a card whose identity is `card_id`
    pub fn update_with_discard_action_of_specific_card(
        &mut self,
        card_deck_index: CardDeckIndex,
        card_id: VariantCardId,
        static_game_data: &StaticGameData,
    ) {
        self.hands[self.active_player_index].remove_card(card_deck_index);
        self.all_hand_bits &= !(1u64 << card_deck_index);
        self.discard_pile.add_card_with_id(card_id);
        let bonus_tokens = static_game_data.variant.bonus_half_clue_tokens_for_discard;
        self.clue_token_bank.add_tokens(bonus_tokens);
        self.deck.reveal_card(card_deck_index, card_id);
    }

    /// Update this [crate::game::game_state::GameState] when the player with the given
    /// `player_index` is given a [Clue] that touches the specified cards
    pub fn update_with_clue_action(
        &mut self,
        card_deck_index: SmallVec<[CardDeckIndex; MAX_HAND_SIZE]>,
        clue: Clue,
        receiver_player_index: usize,
        static_game_data: &StaticGameData,
    ) {
        let clue_empathy = static_game_data.variant.empathy_for_clue(&clue).as_bits();
        for slot in self.hands[receiver_player_index].cards() {
            if card_deck_index.contains(slot) {
                self.clue_touched_cards |= 1 << slot;
                self.deck.update_positive_empathy(*slot, clue_empathy);
            } else {
                self.deck.update_negative_empathy(*slot, clue_empathy);
            }
        }
        self.clue_token_bank
            .use_token()
            .expect("clue given with no tokens");
        self.clues_given += 1;
    }

    /// Return a [VariantCardsBitField] of the cards that can be played successfully in the current
    /// game state
    pub fn playable_cards(&self, static_game_data: &StaticGameData) -> VariantCardsBitField {
        self.playing_stacks.next_cards(&static_game_data.variant)
    }

    pub fn score(&self, variant: &Variant) -> u8 {
        self.playing_stacks.total_size(variant)
    }

    /// Returns true when the game has ended:
    /// - 3 strikes (team lost), or
    /// - score equals theScoremum possible score (team won).
    pub fn is_terminal(&self, static_data: &StaticGameData) -> bool {
        let max_score = static_data.variant.number_of_suits * static_data.variant.stacks_size;
        self.strike_tokens >= 3 || self.score(&static_data.variant) >= max_score
    }

    /// Pace = score + cards_in_deck + num_players - max_theoretical_score.
    /// Positive pace means the team has "breathing room"; negative means they're behind.
    pub fn pace(&self, static_game_data: &StaticGameData) -> i32 {
        let max_score = (static_game_data.variant.number_of_suits
            * static_game_data.variant.stacks_size) as i32;
        self.score(&static_game_data.variant) as i32
            + self.deck.current_size as i32
            + static_game_data.number_of_players as i32
            - max_score
    }

    /// Required efficiency = (cards still needed − live setups) / spare turns.
    ///
    /// "Live setups" are clue-touched cards still in hands — they are already lined up and reduce
    /// the future clue burden.
    ///
    /// Returns 0.0 when the game is already won; f32::INFINITY when no spare turns remain, but
    /// cards are still needed.
    pub fn required_efficiency(&self, static_game_data: &StaticGameData) -> f32 {
        let max_score = (static_game_data.variant.number_of_suits
            * static_game_data.variant.stacks_size) as i32;
        let num_players = static_game_data.number_of_players as usize;
        let hands = &self.hands[..num_players];
        let hand_cards: i32 = hands.iter().map(|h| h.cards().len() as i32).sum();
        let remaining = self.deck.current_size as i32 + hand_cards;
        let still_to_play = max_score - self.score(&static_game_data.variant) as i32;
        let spare_turns = (remaining - still_to_play).max(0);
        if still_to_play <= 0 {
            return 0.0;
        }
        if spare_turns == 0 {
            return f32::INFINITY;
        }
        let all_hand_bits = self.all_hand_bits;
        let live_setups = (self.clue_touched_cards & all_hand_bits).count_ones() as i32;
        (still_to_play - live_setups).max(0) as f32 / spare_turns as f32
    }
}

#[cfg(any(test, feature = "test-support"))]
pub mod unit_test_constants {
    pub mod no_variant_constants {
        use crate::game::card::copies_counting_card_collection::CopiesCountingCardCollection;
        use crate::game::clue_token_bank::ClueTokenBank;
        use crate::game::deck::Deck;
        use crate::game::hand::Hand;
        use crate::game::playing_stacks::PlayingStacks;
        use crate::game::state::table_state::TableState;
        use crate::game::static_game_data::StaticGameData;
        use crate::game::variant::test_variants::NO_VARIANT;

        pub const NOVAR_5_PLAYERS_STATIC_GAME_DATA: StaticGameData = StaticGameData {
            number_of_players: 5,
            variant: NO_VARIANT,
        };

        pub fn initial_five_players_table_state() -> TableState {
            let clue_token_bank = ClueTokenBank::new(16);
            let deck = Deck::new(&NO_VARIANT);
            let hand1 = Hand::new(&vec![]);
            let hand2 = Hand::new(&vec![]);
            let hand3 = Hand::new(&vec![]);
            let hand4 = Hand::new(&vec![]);
            let hand5 = Hand::new(&vec![]);
            let hands = [hand1, hand2, hand3, hand4, hand5, Hand::empty()];
            let playing_stacks = PlayingStacks::empty();
            let strike_tokens = 0;
            let discard_pile = CopiesCountingCardCollection::default();
            TableState::from_parts(
                clue_token_bank,
                deck,
                hands,
                0, // active_player_index
                0, // turn_counter
                playing_stacks,
                strike_tokens,
                discard_pile,
            )
        }

        /// 3-player state with empty playing stacks.
        pub fn empty_stacks_table_state(active_player_index: usize) -> TableState {
            let clue_token_bank = ClueTokenBank::new(10);
            let deck = Deck::new(&NO_VARIANT);
            let mut hands = Hand::empty_array();
            hands[0] = Hand::new(&vec![0, 1, 2, 3]);
            hands[1] = Hand::new(&vec![5, 6, 7, 8, 9]);
            hands[2] = Hand::new(&vec![10, 11, 12, 13]);
            let playing_stacks = PlayingStacks::empty();
            let discard_pile = CopiesCountingCardCollection::empty();
            TableState {
                clue_token_bank,
                clue_touched_cards: 0,
                clues_given: 0,
                deck,
                all_hand_bits: (0..=3u64)
                    .chain(5..=9)
                    .chain(10..=13)
                    .fold(0u64, |acc, i| acc | (1 << i)),
                hands,
                active_player_index,
                current_turn: 0,
                playing_stacks,
                strike_tokens: 0,
                discard_pile,
            }
        }

        /// 3-player state with B1–B4 on the playing stacks.
        pub fn stacked_table_state(active_player_index: usize) -> TableState {
            let clue_token_bank = ClueTokenBank::new(10);
            let deck = Deck::new(&NO_VARIANT);
            let mut hands = Hand::empty_array();
            hands[0] = Hand::new(&vec![0, 1, 2, 3, 4]);
            hands[1] = Hand::new(&vec![5, 6, 7, 8, 9]);
            hands[2] = Hand::new(&vec![10, 11, 12, 13]);
            let mut playing_stacks = PlayingStacks::empty();
            playing_stacks.add_card(15, &NO_VARIANT);
            playing_stacks.add_card(16, &NO_VARIANT);
            playing_stacks.add_card(17, &NO_VARIANT);
            playing_stacks.add_card(18, &NO_VARIANT);
            let discard_pile = CopiesCountingCardCollection::empty();
            TableState {
                clue_token_bank,
                clue_touched_cards: 0,
                clues_given: 0,
                deck,
                all_hand_bits: (0..=4u64)
                    .chain(5..=9)
                    .chain(10..=13)
                    .fold(0u64, |acc, i| acc | (1 << i)),
                hands,
                active_player_index,
                current_turn: 0,
                playing_stacks,
                strike_tokens: 0,
                discard_pile,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::unit_test_constants::no_variant_constants::{
        empty_stacks_table_state, stacked_table_state,
    };
    use super::*;
    use crate::game::variant::test_variants::NO_VARIANT;

    #[test]
    fn should_be_correctly_updated_after_a_draw() {
        let active_player_index = 0;
        let mut game_state = empty_stacks_table_state(active_player_index);
        let drawn_card_deck_index = 14;

        game_state.update_with_draw_action(drawn_card_deck_index);

        let player_hand = &game_state.hands[active_player_index];
        assert!(player_hand.cards().contains(&drawn_card_deck_index));
    }

    #[test]
    fn should_be_correctly_updated_after_a_play() {
        let active_player_index = 1;
        let mut game_state = empty_stacks_table_state(active_player_index);
        let card_deck_index = 8;

        game_state.update_with_play_action(card_deck_index);

        let player_hand = &game_state.hands[active_player_index];
        assert!(!player_hand.cards().contains(&card_deck_index));
    }

    #[test]
    fn should_be_correctly_updated_after_playing_a_specific_card() {
        let active_player_index = 1;
        let mut game_state = empty_stacks_table_state(active_player_index);
        let static_game_data = StaticGameData {
            number_of_players: 3,
            variant: NO_VARIANT,
        };
        let card_deck_index = 8;
        let variant_card_id = 2;
        game_state.update_with_play_action_of_specific_card(
            card_deck_index,
            variant_card_id,
            &static_game_data,
        );

        let player_hand = &game_state.hands[active_player_index];
        assert!(!player_hand.cards().contains(&card_deck_index));
    }

    #[test]
    fn should_be_adding_a_clue_to_the_bank_when_a_5_is_successfully_played() {
        let mut game_state = stacked_table_state(0);
        let static_game_data = StaticGameData {
            number_of_players: 3,
            variant: NO_VARIANT,
        };
        let card_deck_index = 2;
        let variant_card_id = 19;
        game_state.update_with_play_action_of_specific_card(
            card_deck_index,
            variant_card_id,
            &static_game_data,
        );

        assert_eq!(6, game_state.clue_token_bank.whole_clue_tokens_count());
    }

    #[test]
    fn should_be_adding_a_clue_to_the_bank_when_a_card_is_discarded() {
        let mut game_state = stacked_table_state(0);
        let static_game_data = StaticGameData {
            number_of_players: 3,
            variant: NO_VARIANT,
        };
        let card_deck_index = 2;
        game_state.update_with_discard_action(card_deck_index, &static_game_data);

        assert_eq!(6, game_state.clue_token_bank.whole_clue_tokens_count());
    }

    #[test]
    fn should_increase_the_size_of_the_discard_pile_after_a_card_is_discarded() {
        let mut game_state = stacked_table_state(0);
        let static_game_data = StaticGameData {
            number_of_players: 3,
            variant: NO_VARIANT,
        };
        let card_deck_index = 2;
        game_state.update_with_discard_action(card_deck_index, &static_game_data);

        assert_eq!(1, game_state.discard_pile.size());
    }

    #[test]
    fn should_add_a_discarded_card_to_the_discard_pile() {
        let mut game_state = stacked_table_state(0);
        let static_game_data = StaticGameData {
            number_of_players: 3,
            variant: NO_VARIANT,
        };
        let card_deck_index = 2;
        let card_id = 6;
        game_state.update_with_discard_action_of_specific_card(
            card_deck_index,
            card_id,
            &static_game_data,
        );

        assert!(game_state.discard_pile.contains_card_with_id(card_id));
    }
}
