use crate::game::MAX_SUITS_IN_GAME;
use crate::game::card::{VariantCardId, VariantCardsBitField};
use crate::game::variant::Variant;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct PlayingStack {
    played_cards_count: u8,
}

impl PlayingStack {
    pub fn empty() -> Self {
        PlayingStack {
            played_cards_count: 0,
        }
    }

    pub fn add_card(&mut self) {
        self.played_cards_count += 1;
    }
}

impl Default for PlayingStack {
    fn default() -> Self {
        PlayingStack::empty()
    }
}

/// The playing stacks used during the game.
///
/// Contains one stack per suit; newly played cards are placed on the top of their
/// respective stack according to suit.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PlayingStacks {
    cards: VariantCardsBitField,
    stacks: [PlayingStack; MAX_SUITS_IN_GAME],
}

impl PlayingStacks {
    pub fn empty() -> PlayingStacks {
        PlayingStacks {
            cards: 0,
            stacks: [
                PlayingStack::default(),
                PlayingStack::default(),
                PlayingStack::default(),
                PlayingStack::default(),
                PlayingStack::default(),
                PlayingStack::default(),
            ],
        }
    }

    pub fn new(played_cards: Vec<VariantCardId>, variant: &Variant) -> PlayingStacks {
        let mut playing_stacks = PlayingStacks::empty();
        for card_id in played_cards {
            playing_stacks.add_card(card_id, variant);
        }
        playing_stacks
    }

    /// Add the card with the given `card_id` to its stack and return the updated stack size.
    ///
    /// No validity check is performed.
    pub fn add_card(&mut self, card_id: VariantCardId, variant: &Variant) -> u8 {
        let suit_index = variant.get_card_suit_index(card_id);
        self.stacks[suit_index].add_card();
        self.cards |= 1u64 << card_id;
        self.stacks[suit_index].played_cards_count
    }

    /// Return how many cards of the given suit have been played.
    pub fn stack_size(&self, suit_index: usize) -> u8 {
        self.stacks[suit_index].played_cards_count
    }

    /// Get a bitfield of cards that can be successfully played in the current state.
    pub fn next_cards(&self, variant: &Variant) -> VariantCardsBitField {
        let mut next_cards = 0;
        let stacks_size = variant.stacks_size as usize;
        for (index, stack) in self
            .stacks
            .iter()
            .enumerate()
            .take(variant.number_of_suits as usize)
        {
            let count = stack.played_cards_count as usize;
            if count < stacks_size {
                next_cards |= 1 << (stacks_size * index + count);
            } else if count > stacks_size {
                panic!(
                    "Invalid stack state. Stack #{} has card count: {}",
                    index, stack.played_cards_count,
                );
            }
            // count == stacks_size: stack is complete, no card to play
        }
        next_cards
    }

    pub fn total_size(&self, variant: &Variant) -> u8 {
        self.stacks
            .iter()
            .take(variant.number_of_suits as usize)
            .map(|stack| stack.played_cards_count)
            .sum()
    }

    pub fn contains_card_with_id(&self, card_id: VariantCardId) -> bool {
        (self.cards & (1u64 << card_id)) != 0
    }

    pub fn as_bitfield(&self) -> VariantCardsBitField {
        self.cards
    }
}

impl Default for PlayingStacks {
    fn default() -> Self {
        PlayingStacks::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::variant::test_variants::NO_VARIANT;

    #[test]
    fn should_add_a_card_to_the_correct_stack() {
        let mut stacks = PlayingStacks::default();

        stacks.add_card(6, &NO_VARIANT); // Yellow 2

        assert_eq!(1, stacks.stacks[1].played_cards_count);
        assert!(stacks.contains_card_with_id(6));
    }

    #[test]
    fn should_correctly_compute_the_next_cards_when_the_stacks_are_empty() {
        let stacks = PlayingStacks::default();

        let actual = stacks.next_cards(&NO_VARIANT);
        // Red 1, Yellow 1, Green 1, Blue 1, Purple 1
        let expected: VariantCardsBitField = 1 | 1 << 5 | 1 << 10 | 1 << 15 | 1 << 20;
        assert_eq!(expected, actual);
    }

    #[test]
    fn should_correctly_compute_the_next_cards() {
        let mut stacks = PlayingStacks::default();
        stacks.add_card(0, &NO_VARIANT); // Red 1
        stacks.add_card(1, &NO_VARIANT); // Red 2
        stacks.add_card(15, &NO_VARIANT); // Blue 1
        stacks.add_card(16, &NO_VARIANT); // Blue 2
        stacks.add_card(17, &NO_VARIANT); // Blue 3
        stacks.add_card(20, &NO_VARIANT); // Purple 1

        let actual = stacks.next_cards(&NO_VARIANT);
        // Red 3, Yellow 1, Green 1, Blue 4, Purple 2
        let expected: VariantCardsBitField = 1 << 2 | 1 << 5 | 1 << 10 | 1 << 18 | 1 << 21;
        assert_eq!(expected, actual);
    }
}
