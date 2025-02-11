use crate::game::card::Empathy;
use crate::game::card::hanabi_card::HanabiCard;
use crate::game::{MAX_CARDS_IN_DECK, MAX_CLUE_TYPES, MAX_CLUE_VALUES_PER_TYPE, MAX_UNIQUE_CARDS_IN_DECK};
use crate::game::variant::Variant;

pub struct StaticGameData {
    pub card_copies_count_by_id: [u8; MAX_UNIQUE_CARDS_IN_DECK],
    pub deck_size: usize,
    pub hands_size: u8,
    pub number_of_players: u8,
    pub variant: Variant,
    pub empathy_by_clue: [[Empathy;MAX_CLUE_VALUES_PER_TYPE];MAX_CLUE_TYPES],
}
