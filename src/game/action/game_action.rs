use crate::game::card::CardDeckIndex;
use crate::game::clue::Clue;
use crate::game::state::PlayerIndex;
use smallvec::SmallVec;
use crate::game::MAX_HAND_SIZE;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum GameAction {
    Draw {
        player_index: PlayerIndex,
        card_deck_index: CardDeckIndex,
    },
    Play {
        player_index: PlayerIndex,
        card_deck_index: CardDeckIndex,
    },
    Discard {
        player_index: PlayerIndex,
        card_deck_index: CardDeckIndex,
    },
    Clue {
        player_index: PlayerIndex,
        touched_card_deck_indexes: SmallVec<[CardDeckIndex; MAX_HAND_SIZE]>,
        clue: Clue,
    },
}
