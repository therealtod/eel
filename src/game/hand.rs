use crate::game::{MAX_HAND_SIZE, MAX_PLAYERS_IN_GAME};
use crate::game::card::CardDeckIndex;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
pub struct Hand {
    slots: [CardDeckIndex; MAX_HAND_SIZE],
    len: u8,
}

impl PartialEq for Hand {
    fn eq(&self, other: &Self) -> bool {
        self.cards() == other.cards()
    }
}

impl Eq for Hand {}

impl Hash for Hand {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.cards().hash(state);
    }
}

impl Hand {
    pub fn empty() -> Hand {
        Hand {
            slots: [0; MAX_HAND_SIZE],
            len: 0,
        }
    }

    /// Create a hand from a slice ordered oldest-first (chop first, slot 1 last).
    pub fn new(card_indexes: &[CardDeckIndex]) -> Hand {
        debug_assert!(card_indexes.len() <= MAX_HAND_SIZE);
        let mut hand = Hand::empty();
        for (i, &card) in card_indexes.iter().rev().enumerate() {
            hand.slots[i] = card;
        }
        hand.len = card_indexes.len() as u8;
        hand
    }

    /// Returns the live cards in this hand (newest first / slot 1 first, oldest last).
    pub fn cards(&self) -> &[CardDeckIndex] {
        &self.slots[..self.len as usize]
    }

    /// Adds a card as the newest (slot 1).
    pub fn add_card_to_slot_1(&mut self, card_deck_index: CardDeckIndex) {
        debug_assert!((self.len as usize) < MAX_HAND_SIZE);
        let len = self.len as usize;
        self.slots.copy_within(0..len, 1);
        self.slots[0] = card_deck_index;
        self.len += 1;
    }

    pub fn remove_card(&mut self, card_deck_index: CardDeckIndex) {
        let len = self.len as usize;
        if let Some(pos) = self.slots[..len].iter().position(|&x| x == card_deck_index) {
            self.slots.copy_within(pos + 1..len, pos);
            self.len -= 1;
        }
    }

    /// Remove the card at the given 1-based slot (slot 1 = newest).
    pub fn remove_card_from_slot_from_newest(&mut self, slot_index: usize) {
        let len = self.len as usize;
        let pos = slot_index - 1;
        self.slots.copy_within(pos + 1..len, pos);
        self.len -= 1;
    }

    /// Return the card at the given 1-based slot (slot 1 = newest).
    pub fn card_in_slot(&self, slot_index: usize) -> CardDeckIndex {
        self.slots[slot_index - 1]
    }

    /// Return an array of `MAX_PLAYERS_IN_GAME` empty hands.
    pub fn empty_array() -> [Hand; MAX_PLAYERS_IN_GAME] {
        [
            Hand::empty(),
            Hand::empty(),
            Hand::empty(),
            Hand::empty(),
            Hand::empty(),
            Hand::empty(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_the_given_card_to_the_correct_slot() {
        let mut hand = Hand::new(&vec![3, 15, 2, 12]);

        hand.add_card_to_slot_1(55);

        assert_eq!(Hand::new(&vec![3, 15, 2, 12, 55]), hand);
    }

    #[test]
    fn removes_the_card_from_the_correct_slot() {
        let mut hand = Hand::new(&vec![3, 15, 2, 12, 8]);

        hand.remove_card_from_slot_from_newest(2);

        assert_eq!(Hand::new(&vec![3, 15, 2, 8]), hand);
    }

    #[test]
    fn gets_the_correct_card_from_the_given_slot() {
        let hand = Hand::new(&vec![3, 15, 2, 12, 8]);

        assert_eq!(2, hand.card_in_slot(3));
    }
}
