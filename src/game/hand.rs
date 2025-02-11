use std::collections::LinkedList;
use crate::game::card::card_collection::CardCollection;
use crate::game::card::{Empathy, CardPositionInStartingDeck};

#[derive(PartialEq, Debug)]
pub struct Hand {
    size: usize,
    pub slots: Vec<usize>,
}

impl Hand {
    pub fn add_card_to_slot_1(&mut self, card_id: CardPositionInStartingDeck) {
        self.slots.push(card_id);
    }

    pub fn remove_card(&mut self, card_id: CardPositionInStartingDeck) {
        // TODO: check if element is part of the vec
        self.slots.retain(|&x| x != card_id);
    }

    pub fn remove_card_from_slot(&mut self, slot_index: usize) {
        self.slots.remove(self.size - slot_index);
    }

    pub fn get_card_in_slot(&self, slot_index: usize) -> usize {
        self.slots[self.size - slot_index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_the_given_card_to_the_correct_slot() {
        let size = 5;
        let slots = vec![3, 15, 2, 12];
        let mut hand = Hand { size, slots };

        hand.add_card_to_slot_1(55);

        let expected_slots = vec![3, 15, 2, 12, 55];
        let expected = Hand { size, slots: expected_slots };

        assert_eq!(expected, hand);
    }
    #[test]
    fn removes_the_card_from_the_correct_slot() {
        let size = 5;
        let slots = vec![3, 15, 2, 12, 8];
        let mut hand = Hand { size, slots };

        hand.remove_card_from_slot(2);

        let expected_slots = vec![3, 15, 2, 8];
        let expected = Hand { size, slots: expected_slots };

        assert_eq!(expected, hand);
    }

    #[test]
    fn gets_the_correct_card_from_the_given_slot() {
        let size = 5;
        let slots = vec![3, 15, 2, 12, 8];
        let hand = Hand { size, slots };

        let expected = 2;
        let actual = hand.get_card_in_slot(3);

        assert_eq!(expected, actual);
    }
}
