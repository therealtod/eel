use crate::game::card::CardDeckIndex;
use crate::game::clue::Clue;
use crate::game::state::PlayerIndex;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum GameAction {
    Draw { player_index: PlayerIndex, card_deck_index: CardDeckIndex },
    Play { player_index: PlayerIndex, card_deck_index: CardDeckIndex },
    Discard { player_index: PlayerIndex, card_deck_index: CardDeckIndex },
    Clue { player_index: PlayerIndex, touched_card_deck_indexes: Vec<CardDeckIndex>, clue: Clue },
}
