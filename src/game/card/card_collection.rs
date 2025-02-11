use crate::game::card::{CardPositionInStartingDeck, Empathy, UniqueCardId};
use crate::game::MAX_UNIQUE_CARDS_IN_DECK;
use crate::game::variant::Variant;

#[derive(Debug)]
pub struct CardCollection {
    card_copies_count: [u8; MAX_UNIQUE_CARDS_IN_DECK],
}

impl CardCollection {
    pub fn add_card(&mut self, card_id: UniqueCardId) {
        self.card_copies_count[card_id] += 1
    }
}
