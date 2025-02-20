use crate::game::card::{CardDeckIndex, VariantCardId, VariantCardsBitField};
use crate::game::variant::Variant;
use crate::game::{MAX_CARDS_IN_DECK, MAX_UNIQUE_CARDS_IN_DECK};

/// Representation of the deck of a game of Hanabi.
/// 
/// 
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Deck {
    pub current_size: u8,
    total_copies_per_id: [u8; MAX_UNIQUE_CARDS_IN_DECK],
    revealed_copies_per_index: [u8; MAX_UNIQUE_CARDS_IN_DECK],
    // revealed_values: [VariantCardId; MAX_CARDS_IN_DECK],
    empathy_by_index: [VariantCardsBitField; MAX_CARDS_IN_DECK],
    revealed_indexes: u64,
}

impl Deck {
    pub fn new(
        variant: &Variant,
    ) -> Self {
        Deck {
            current_size: variant.deck_size,
            total_copies_per_id: variant.card_copies_count_by_id,
            revealed_copies_per_index: [0; MAX_UNIQUE_CARDS_IN_DECK],
            empathy_by_index: [VariantCardsBitField::MAX; MAX_CARDS_IN_DECK],
            revealed_indexes: 0,
        }
    }

    pub fn of(
        current_size: u8,
        total_copies_per_id: [u8; MAX_UNIQUE_CARDS_IN_DECK],
        revealed_copies_per_index: [u8; MAX_UNIQUE_CARDS_IN_DECK],
        empathy_by_index: [VariantCardsBitField; MAX_CARDS_IN_DECK],
        revealed_indexes: u64,
    ) -> Self {
        Deck {
            current_size,
            total_copies_per_id,
            revealed_copies_per_index,
            empathy_by_index,
            revealed_indexes,
        }
    }

    /// Decrement the size this deck by the given amount
    pub fn decrement_size(&mut self, amount: u8) {
        self.current_size -= amount;
    }

    /// Decrement the size of this deck by one
    pub fn decrement_size_by_one(&mut self) {
        self.current_size -= 1;
    }
    pub fn get_global_empathy(&self, deck_card_index: CardDeckIndex) -> VariantCardsBitField {
        self.empathy_by_index[deck_card_index as usize]
    }
    pub fn reveal_card(
        &mut self,
        card_position_in_starting_deck: CardDeckIndex,
        card_id: VariantCardId,
    ) {
        let card_empathy = 1 << card_id;
        // self.revealed_values[card_position_in_starting_deck] = card_id;
        self.revealed_copies_per_index[card_id] += 1;
        self.empathy_by_index[card_position_in_starting_deck as usize] = card_empathy;
        self.revealed_indexes |= 1 << card_position_in_starting_deck;
        if self.revealed_copies_per_index[card_id] == self.total_copies_per_id[card_id] {
            // TODO: This is a shit
            let mut revealed_cards: Vec<(CardDeckIndex, VariantCardId)> = Vec::with_capacity(MAX_UNIQUE_CARDS_IN_DECK);
            for (index, empathy)
            in &mut self.empathy_by_index.iter_mut().enumerate() {
                if self.revealed_indexes & (1 << index) == 0 {
                    let new_empathy = *empathy & !card_empathy;
                    if new_empathy.count_ones() == 1 && empathy.count_ones() != 1 {
                        revealed_cards.push((index as CardDeckIndex, new_empathy.trailing_zeros() as VariantCardId))
                    }
                    *empathy = new_empathy;
                }
            }
            for (index, new_card_revealed_id) in revealed_cards {
                self.reveal_card(index, new_card_revealed_id);
            }
        }
    }

    pub fn update_positive_empathy(
        &mut self,
        card_position_in_starting_deck: &CardDeckIndex,
        empathy_update: VariantCardsBitField,
    ) {
        let pos = *card_position_in_starting_deck as usize;
        self.empathy_by_index[pos] &= empathy_update;
        let new_empathy = self.empathy_by_index[pos];
        if new_empathy.count_ones() == 1 {
            let revealed_card_id: VariantCardId = new_empathy.trailing_zeros() as VariantCardId;
            self.reveal_card(*card_position_in_starting_deck, revealed_card_id);
        }
    }

    pub fn update_negative_empathy(
        &mut self,
        card_position_in_starting_deck: &CardDeckIndex,
        empathy_update: VariantCardsBitField,
    ) {
        let pos = *card_position_in_starting_deck as usize;
        self.empathy_by_index[pos] &= !empathy_update;
        let new_empathy = self.empathy_by_index[pos];
        if new_empathy.count_ones() == 1 {
            let revealed_card_id: VariantCardId = new_empathy.trailing_zeros() as VariantCardId;
            self.reveal_card(*card_position_in_starting_deck, revealed_card_id);
        }
    }
}

#[cfg(test)]
pub mod unit_test_constant {
    use crate::game::deck::Deck;
    use crate::game::{ALL_CARDS_MASK, MAX_CARDS_IN_DECK, MAX_UNIQUE_CARDS_IN_DECK};
    use crate::game::deck::unit_test_constant::novariant_constants::COPIES_COUNT_BY_ID;

    pub mod novariant_constants {
        use crate::game::card::{VariantCardId, VariantCardsBitField};
        use crate::game::MAX_UNIQUE_CARDS_IN_DECK;

        pub const COPIES_COUNT_BY_ID: [u8; MAX_UNIQUE_CARDS_IN_DECK] = [
            3, 2, 2, 2, 1, 3, 2, 2, 2, 1, 3, 2, 2, 2, 1, 3, 2, 2, 2, 1, 3, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ];
        
        #[repr(usize)]
        #[derive(Copy, Clone)]
        pub enum NoVarCards {
            R1,
            R2,
            R3,
            R4,
            R5,
            Y1,
            Y2,
            Y3,
            Y4,
            Y5,
            G1,
            G2,
            G3,
            G4,
            G5,
            B1,
            B2,
            B3,
            B4,
            B5,
            P1,
            P2,
            P3,
            P4,
            P5,
        }
        
        impl NoVarCards {
            pub fn as_variant_card_id(&self) -> VariantCardId {
                *self as VariantCardId
            }
        }
        
        pub const R1_MASK: VariantCardsBitField = 1;
        pub const R2_MASK: VariantCardsBitField = 2;
        pub const R3_MASK: VariantCardsBitField = 1 << 2;
        pub const R4_MASK: VariantCardsBitField = 1 << 3;
        pub const R5_MASK: VariantCardsBitField = 1 << 4;
        pub const Y1_MASK: VariantCardsBitField = 1 << 5;
        pub const Y2_MASK: VariantCardsBitField = 1 << 6;
        pub const Y3_MASK: VariantCardsBitField = 1 << 7;
        pub const Y4_MASK: VariantCardsBitField = 1 << 8;
        pub const Y5_MASK: VariantCardsBitField = 1 << 9;
        pub const G1_MASK: VariantCardsBitField = 1 << 10;
        pub const G2_MASK: VariantCardsBitField = 1 << 11;
        pub const G3_MASK: VariantCardsBitField = 1 << 12;
        pub const G4_MASK: VariantCardsBitField = 1 << 13;
        pub const G5_MASK: VariantCardsBitField = 1 << 14;
        pub const B1_MASK: VariantCardsBitField = 1 << 15;
        pub const B2_MASK: VariantCardsBitField = 1 << 16;
        pub const B3_MASK: VariantCardsBitField = 1 << 17;
        pub const B4_MASK: VariantCardsBitField = 1 << 18;
        pub const B5_MASK: VariantCardsBitField = 1 << 19;
        pub const P1_MASK: VariantCardsBitField = 1 << 20;
        pub const P2_MASK: VariantCardsBitField = 1 << 21;
        pub const P3_MASK: VariantCardsBitField = 1 << 22;
        pub const P4_MASK: VariantCardsBitField = 1 << 23;
        pub const P5_MASK: VariantCardsBitField = 1 << 24;
    }

    pub const NEW_DECK: Deck = Deck {
        current_size: 50,
        total_copies_per_id: COPIES_COUNT_BY_ID,
        revealed_copies_per_index: [0; MAX_UNIQUE_CARDS_IN_DECK],
        empathy_by_index: [ALL_CARDS_MASK; MAX_CARDS_IN_DECK],
        revealed_indexes: 0,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::deck::tests::unit_test_constant::NEW_DECK;
    use crate::game::deck::tests::unit_test_constant::novariant_constants::*;
    
    #[test]
    fn should_decrement_size_when_drawing_a_card() {
        let mut deck = NEW_DECK.clone();
        
        deck.decrement_size(3);
        
        assert_eq!(47, deck.current_size)
    }


    #[test]
    fn should_update_empathy_of_revealed_index() {
        let mut deck = NEW_DECK.clone();

        deck.reveal_card(42, 2);
        let expected = R3_MASK;
        let actual = deck.empathy_by_index[42];
        assert_eq!(expected, actual);
    }

    #[test]
    fn should_update_empathy_indirectly_when_all_copies_are_revealed() {
        let mut deck = NEW_DECK.clone();

        deck.reveal_card(42, 2);
        deck.reveal_card(22, 2);
        assert_eq!(R3_MASK, deck.empathy_by_index[42]);
        assert_eq!(R3_MASK, deck.empathy_by_index[22]);
        assert_eq!(VariantCardsBitField::MAX & !R3_MASK, deck.empathy_by_index[1]);
    }

    #[test]
    fn should_recursively_update_empathy() {
        let mut deck = Deck {
            current_size: 50,
            total_copies_per_id: COPIES_COUNT_BY_ID,
            revealed_copies_per_index: [
                3, 2, 2, 2, 1, 3, 2, 1, 2, 0, 3, 2, 2, 2, 1, 3, 2, 2, 2, 1, 3, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ],
            // revealed_values: [0; MAX_CARDS_IN_DECK],
            empathy_by_index: [
                R1_MASK,
                R1_MASK,
                R1_MASK,
                R2_MASK,
                R2_MASK,
                R3_MASK,
                R3_MASK,
                R4_MASK,
                R4_MASK,
                R5_MASK,
                Y1_MASK,
                Y1_MASK,
                Y1_MASK,
                Y2_MASK,
                Y2_MASK,
                Y3_MASK,
                Y4_MASK,
                Y4_MASK,
                G1_MASK,
                G1_MASK,
                G1_MASK,
                G2_MASK,
                G2_MASK,
                G3_MASK,
                G3_MASK,
                G4_MASK,
                G4_MASK,
                G5_MASK,
                B1_MASK,
                B1_MASK,
                B1_MASK,
                B2_MASK,
                B2_MASK,
                B3_MASK,
                B3_MASK,
                B4_MASK,
                B4_MASK,
                B5_MASK,
                P1_MASK,
                P1_MASK,
                P1_MASK,
                P2_MASK,
                P2_MASK,
                P3_MASK,
                P3_MASK,
                P4_MASK,
                P4_MASK,
                Y3_MASK | Y5_MASK,
                Y3_MASK | Y5_MASK,
                Y5_MASK | P5_MASK,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
            revealed_indexes: 0,
        };

        deck.reveal_card(47, 7);
        assert_eq!(Y3_MASK, deck.empathy_by_index[47]);
        assert_eq!(Y5_MASK, deck.empathy_by_index[48]);
        assert_eq!(P5_MASK, deck.empathy_by_index[49]);
    }
}
