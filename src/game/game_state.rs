use crate::game::card::{Empathy, CardPositionInStartingDeck, UniqueCardId};
use crate::game::clue::Clue;

pub trait GameState {
    fn update_with_draw_action(
        &mut self, player_index: usize,
        card_position_in_starting_deck: CardPositionInStartingDeck
    );
    fn update_with_play_action(
        &mut self,
        player_index: usize,
        card_position_in_starting_deck: CardPositionInStartingDeck,
    );
    fn update_with_play_action_of_specific_card(
        &mut self,
        player_index: usize,
        card_position_in_starting_deck: CardPositionInStartingDeck,
        card_id: UniqueCardId,
    );
    fn update_with_discard_action(
        &mut self,
        player_index: usize,
        card_position_in_starting_deck: CardPositionInStartingDeck,
    );
    fn update_with_discard_action_of_specific_card(
        &mut self,
        player_index: usize,
        card_position_in_starting_deck: CardPositionInStartingDeck,
        card_id: UniqueCardId,
    );
    fn update_with_clue_action(
        &mut self,
        card_positions_in_starting_deck: Vec<CardPositionInStartingDeck>,
        clue: Clue,
        receiver_player_index: usize,
    );
}
