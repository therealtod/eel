use std::env::var;
use crate::game::card::{Empathy, CardPositionInStartingDeck, UniqueCardId};
use crate::game::card::card_collection::CardCollection;
use crate::game::card::hanabi_card::HanabiCard;
use crate::game::clue::Clue;
use crate::game::clue_token_bank::ClueTokenBank;
use crate::game::deck::Deck;
use crate::game::game_state::GameState;
use crate::game::hand::Hand;
use crate::game::playing_stacks::PlayingStacks;
use crate::game::static_game_data::StaticGameData;

pub struct CoreGameState {
    cards_left_in_deck: i32,
    clue_token_bank: ClueTokenBank,
    deck: Deck,
    hands: [Hand; 6],
    playing_stacks: PlayingStacks,
    public_empathy: [Empathy; 60],
    static_game_data: StaticGameData,
    strike_tokens: u8,
    trash_pile: CardCollection,
}

impl GameState for CoreGameState {
    fn update_with_draw_action(
        &mut self, player_index: usize,
        card_position_in_starting_deck: CardPositionInStartingDeck,
    ) {
        self.hands[player_index].add_card_to_slot_1(card_position_in_starting_deck);
        self.cards_left_in_deck -= 1;
    }

    fn update_with_play_action(
        &mut self,
        player_index: usize,
        card_position_in_starting_deck: CardPositionInStartingDeck,
    ) {
        self.hands[player_index].remove_card(card_position_in_starting_deck);
    }

    fn update_with_play_action_of_specific_card(
        &mut self,
        player_index: usize,
        card_position_in_starting_deck: CardPositionInStartingDeck,
        card_id: UniqueCardId,
    ){
        Self::update_with_play_action(self, player_index, card_position_in_starting_deck);
        self.deck.reveal_card(card_position_in_starting_deck, card_id);
        let (success, bonus_tokens) = self.playing_stacks.play(
            card_id,
            &self.static_game_data.variant,
        );
        self.clue_token_bank.add_tokens(bonus_tokens);
        if !success {
            self.strike_tokens += 1
        }
    }

    fn update_with_discard_action(
        &mut self,
        player_index: usize,
        card_position_in_starting_deck: CardPositionInStartingDeck,
    ) {
        self.hands[player_index].remove_card(card_position_in_starting_deck);
        let bonus_tokens = self.static_game_data.variant.get_bonus_half_tokens_for_discarding();
        self.clue_token_bank.add_tokens(bonus_tokens);
    }

    fn update_with_discard_action_of_specific_card(
        &mut self,
        player_index: usize,
        card_position_in_starting_deck: CardPositionInStartingDeck,
        card_id: UniqueCardId,
    ) {
        self.update_with_draw_action(player_index, card_position_in_starting_deck);
        self.trash_pile.add_card(card_id);
        self.deck.reveal_card(card_position_in_starting_deck, card_id)
    }

    fn update_with_clue_action(
        &mut self,
        card_positions_in_starting_deck: Vec<CardPositionInStartingDeck>,
        clue: Clue,
        receiver_player_index: usize,
    ) {
        let clue_empathy = self
            .static_game_data.empathy_by_clue[clue.clue_type][clue.clue_value];
        for slot in &self.hands[receiver_player_index].slots {
            if card_positions_in_starting_deck.contains(slot) {
                self.deck.update_positive_empathy(slot, clue_empathy);
            } else {
                self.deck.update_negative_empathy(slot, clue_empathy);
            }
        }
    }
}