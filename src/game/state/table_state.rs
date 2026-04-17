#[cfg(test)]
use mockall::automock;
use crate::game::card::{CardDeckIndex, VariantCardId, VariantCardsBitField};
use crate::game::card::copies_counting_card_collection::CopiesCountingCardCollection;
use crate::game::card::DeckCardsBitField;
use crate::game::clue::Clue;
use crate::game::clue_token_bank::ClueTokenBank;
use crate::game::deck::Deck;
use crate::game::hand::Hand;
use crate::game::playing_stacks::PlayingStacks;
use crate::game::static_game_data::StaticGameData;
use crate::game::MAX_PLAYERS_IN_GAME;

/// The state of the table of a game of Hanabi
///
/// It captures a snapshot of the state of the table, and keeps of  the information on the 
/// various game entities: playing stacks, trash pile, etc...
///
/// `StaticGameData` is intentionally *not* stored here: it never changes over a game, so
/// cloning it into every alpha-beta node would be wasteful.  Instead, callers hold a single
/// `StaticGameData` and pass `&StaticGameData` to the methods that require it.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TableState {
    pub clue_token_bank: ClueTokenBank,
    pub clue_touched_cards: DeckCardsBitField,
    pub deck: Deck,
    pub hands: [Hand; MAX_PLAYERS_IN_GAME],
    pub player_on_turn_index: usize,
    pub playing_stacks: PlayingStacks,
    pub strike_tokens: u8,
    pub discard_pile: CopiesCountingCardCollection,
}

#[cfg_attr(test, automock)]
impl TableState {
    /// Construct a `TableState` from all its constituent parts.
    ///
    /// Intended for use in tests and scenario loading.
    pub fn from_parts(
        clue_token_bank: ClueTokenBank,
        deck: Deck,
        hands: [Hand; MAX_PLAYERS_IN_GAME],
        player_on_turn_index: usize,
        playing_stacks: PlayingStacks,
        strike_tokens: u8,
        discard_pile: CopiesCountingCardCollection,
    ) -> Self {
        TableState {
            clue_token_bank,
            clue_touched_cards: 0,
            deck,
            hands,
            player_on_turn_index,
            playing_stacks,
            strike_tokens,
            discard_pile,
        }
    }

    pub fn new(static_game_data: &StaticGameData) -> Self {
        let clue_token_bank = ClueTokenBank::default();
        let deck = Deck::new(&static_game_data.variant);
        let hands = [
            Hand::empty(),
            Hand::empty(),
            Hand::empty(),
            Hand::empty(),
            Hand::empty(),
            Hand::empty(),
        ];
        let player_on_turn_index = 0;
        let playing_stacks = PlayingStacks::empty();
        let strike_tokens = 0;
        let trash_pile = CopiesCountingCardCollection::empty();
        TableState {
            clue_token_bank,
            clue_touched_cards: 0,
            deck,
            hands,
            player_on_turn_index,
            playing_stacks,
            strike_tokens,
            discard_pile: trash_pile,
        }
    }

    /// Update this [crate::game::game_state::GameState] when the given card is drawn by the player 
    /// with the given `player index` 
    pub fn update_with_draw_action(
        &mut self,
        card_deck_index: CardDeckIndex,
    ) {
        self.hands[self.player_on_turn_index].add_card_to_slot_1(card_deck_index);
        self.deck.decrement_size_by_one()
    }

    /// Update this [crate::game::game_state::GameState] when the player with the given 
    /// `player_index` tries to play a card
    pub fn update_with_play_action(
        &mut self,
        card_deck_index: CardDeckIndex,
    ) {
        self.hands[self.player_on_turn_index].remove_card(card_deck_index);
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
            let new_stack_size = self.playing_stacks.add_card(card_id, &static_game_data.variant);
            let bonus_clue_tokens = if new_stack_size == static_game_data.variant.stacks_size {
                static_game_data.variant.bonus_half_clue_tokens_for_completing_stack
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
        self.hands[self.player_on_turn_index].remove_card(card_deck_index);
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
        self.hands[self.player_on_turn_index].remove_card(card_deck_index);
        self.discard_pile.add_card_with_id(card_id);
        let bonus_tokens = static_game_data.variant.bonus_half_clue_tokens_for_discard;
        self.clue_token_bank.add_tokens(bonus_tokens);
        self.deck.reveal_card(card_deck_index, card_id);
    }

    /// Update this [crate::game::game_state::GameState] when the player with the given 
    /// `player_index` is given a [Clue] that touches the specified cards
    pub fn update_with_clue_action(
        &mut self,
        card_deck_index: Vec<CardDeckIndex>,
        clue: Clue,
        receiver_player_index: usize,
        static_game_data: &StaticGameData,
    ) {
        let clue_empathy = static_game_data.variant.empathy_for_clue(&clue);
        for slot in self.hands[receiver_player_index].cards() {
            if card_deck_index.contains(slot) {
                self.clue_touched_cards |= 1 << slot;
                self.deck.update_positive_empathy(slot, clue_empathy);
            } else {
                self.deck.update_negative_empathy(slot, clue_empathy);
            }
        }
    }

    /// Return a [VariantCardsBitField] of the cards that can be played successfully in the current
    /// game state
    pub fn playable_cards(&self, static_game_data: &StaticGameData) -> VariantCardsBitField {
        self.playing_stacks.next_cards(&static_game_data.variant)
    }
t
    pub fn score(&self) -> u8 {
        self.playing_stacks.total_size()
    }
}

#[cfg(test)]
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
            let hand1 = Hand::new(vec![]);
            let hand2 = Hand::new(vec![]);
            let hand3 = Hand::new(vec![]);
            let hand4 = Hand::new(vec![]);
            let hand5 = Hand::new(vec![]);
            let hands = [hand1, hand2, hand3, hand4, hand5, Hand::empty()];
            let playing_stacks = PlayingStacks::empty();
            let strike_tokens = 0;
            let trash_pile = CopiesCountingCardCollection::default();
            TableState {
                clue_token_bank,
                clue_touched_cards: 0,
                deck,
                hands,
                player_on_turn_index: 0,
                playing_stacks,
                strike_tokens,
                discard_pile: trash_pile,
            }
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::card::copies_counting_card_collection::CopiesCountingCardCollection;
    use crate::game::variant::test_variants::NO_VARIANT;

    #[test]
    fn should_be_correctly_updated_after_a_draw() {
        let player_on_turn_index = 0;
        let mut game_state = empty_stacks_test_state(player_on_turn_index);
        let drawn_card_deck_index = 14;
        
        game_state.update_with_draw_action(drawn_card_deck_index);
        
        let player_hand = &game_state.hands[player_on_turn_index];
        assert!(player_hand.cards().contains(&drawn_card_deck_index));
    }

    #[test]
    fn should_be_correctly_updated_after_a_play() {
        let player_on_turn_index = 1;
        let mut game_state = empty_stacks_test_state(player_on_turn_index);
        let card_deck_index = 8;

        game_state.update_with_play_action(card_deck_index);

        let player_hand = &game_state.hands[player_on_turn_index];
        assert!(!player_hand.cards().contains(&card_deck_index));
    }

    #[test]
    fn should_be_correctly_updated_after_playing_a_specific_card() {
        let player_on_turn_index = 1;
        let mut game_state = empty_stacks_test_state(player_on_turn_index);
        let static_game_data = StaticGameData { number_of_players: 3, variant: NO_VARIANT };
        let card_deck_index = 8;
        let variant_card_id = 2;
        game_state.update_with_play_action_of_specific_card(
          card_deck_index,
          variant_card_id,
          &static_game_data,
        );

        let player_hand = &game_state.hands[player_on_turn_index];
        assert!(!player_hand.cards().contains(&card_deck_index));
    }

    #[test]
    fn should_be_adding_a_clue_to_the_bank_when_a_5_is_successfully_played() {
        let mut game_state = test_state(0);
        let static_game_data = StaticGameData { number_of_players: 3, variant: NO_VARIANT };
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
        let mut game_state = test_state(0);
        let static_game_data = StaticGameData { number_of_players: 3, variant: NO_VARIANT };
        let card_deck_index = 2;
        game_state.update_with_discard_action(card_deck_index, &static_game_data);

        assert_eq!(6, game_state.clue_token_bank.whole_clue_tokens_count());
    }

    #[test]
    fn should_increase_the_size_of_the_discard_pile_after_a_card_is_discarded() {
        let mut game_state = test_state(0);
        let static_game_data = StaticGameData { number_of_players: 3, variant: NO_VARIANT };
        let card_deck_index = 2;
        game_state.update_with_discard_action(card_deck_index, &static_game_data);

        assert_eq!(1, game_state.discard_pile.size());
    }
    
    #[test]
    fn should_add_a_discarded_card_to_the_discard_pile() {
        let mut game_state = test_state(0);
        let static_game_data = StaticGameData { number_of_players: 3, variant: NO_VARIANT };
        let card_deck_index = 2;
        let card_id = 6;
        game_state.update_with_discard_action_of_specific_card(card_deck_index, card_id, &static_game_data);

        assert!(game_state.discard_pile.contains_card_with_id(card_id));
    }

    fn empty_stacks_test_state(player_on_turn_index: usize) -> TableState {
        let clue_token_bank = ClueTokenBank::new(10);
        let deck = Deck::new(&NO_VARIANT);
        let hand1 = Hand::new(vec![0, 1, 2, 3]);
        let hand2 = Hand::new(vec![5, 6, 7, 8, 9]);
        let hand3 = Hand::new(vec![10, 11, 12, 13]);
        let hands = [hand1, hand2, hand3, Hand::empty(), Hand::empty(), Hand::empty()];
        let playing_stacks = PlayingStacks::empty();
        let strike_tokens = 0;
        let trash_pile = CopiesCountingCardCollection::empty();
        TableState {
            clue_token_bank,
            clue_touched_cards: 0,
            deck,
            hands,
            player_on_turn_index,
            playing_stacks,
            strike_tokens,
            discard_pile: trash_pile,
        }
    }

    fn test_state(player_on_turn_index: usize) -> TableState {
        let clue_token_bank = ClueTokenBank::new(10);
        let deck = Deck::new(&NO_VARIANT);
        let hand1 = Hand::new(vec![0, 1, 2, 3, 4]);
        let hand2 = Hand::new(vec![5, 6, 7, 8, 9]);
        let hand3 = Hand::new(vec![10, 11, 12, 13]);
        let hands = [hand1, hand2, hand3, Hand::empty(), Hand::empty(), Hand::empty()];
        let mut playing_stacks = PlayingStacks::empty();
        playing_stacks.add_card(15, &NO_VARIANT);
        playing_stacks.add_card(16, &NO_VARIANT);
        playing_stacks.add_card(17, &NO_VARIANT);
        playing_stacks.add_card(18, &NO_VARIANT);
        let strike_tokens = 0;
        let trash_pile = CopiesCountingCardCollection::empty();
        TableState {
            clue_token_bank,
            clue_touched_cards: 0,
            deck,
            hands,
            player_on_turn_index,
            playing_stacks,
            strike_tokens,
            discard_pile: trash_pile,
        }
    }
}
