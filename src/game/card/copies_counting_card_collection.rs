use crate::game::MAX_UNIQUE_CARDS_IN_DECK;
use crate::game::card::VariantCardId;

/// Tracks how many copies of each card are in the collection.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct CopiesCountingCardCollection {
    card_copies_count: [u8; MAX_UNIQUE_CARDS_IN_DECK],
    size: u8,
}

impl CopiesCountingCardCollection {
    #[must_use]
    pub fn empty() -> Self {
        CopiesCountingCardCollection {
            card_copies_count: [0; MAX_UNIQUE_CARDS_IN_DECK],
            size: 0,
        }
    }

    pub fn add_card(&mut self) {
        self.size += 1;
    }

    pub fn set_size(&mut self, size: u8) {
        self.size = size;
    }

    pub fn add_card_with_id(&mut self, card_id: VariantCardId) {
        self.card_copies_count[card_id] += 1;
        self.size += 1;
    }

    #[must_use]
    pub fn contains_card_with_id(&self, card_id: VariantCardId) -> bool {
        self.card_copies_count[card_id] > 0
    }

    #[must_use]
    pub fn copies_of(&self, card_id: VariantCardId) -> u8 {
        self.card_copies_count[card_id]
    }

    #[must_use]
    pub fn size(&self) -> u8 {
        self.size
    }
}

impl Default for CopiesCountingCardCollection {
    fn default() -> Self {
        Self::empty()
    }
}
