use crate::game::card::{CardDeckIndex, CardIdentityMask, VariantCardId, VariantCardsBitField};
use crate::game::variant::Variant;
use crate::game::{MAX_CARDS_IN_DECK, MAX_UNIQUE_CARDS_IN_DECK};
use smallvec::SmallVec;

/// Representation of the deck of a game of Hanabi.
///
///
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Deck {
    pub current_size: u8,
    total_copies_per_id: [u8; MAX_UNIQUE_CARDS_IN_DECK],
    revealed_copies_per_index: [u8; MAX_UNIQUE_CARDS_IN_DECK],
    empathy_by_index: [CardIdentityMask; MAX_CARDS_IN_DECK],
    revealed_indexes: u64,
}

impl Deck {
    #[must_use]
    pub fn new(variant: &Variant) -> Self {
        let unknown = CardIdentityMask::all(variant);
        Deck {
            current_size: variant.deck_size,
            total_copies_per_id: variant.card_copies_count_by_id,
            revealed_copies_per_index: [0; MAX_UNIQUE_CARDS_IN_DECK],
            empathy_by_index: [unknown; MAX_CARDS_IN_DECK],
            revealed_indexes: 0,
        }
    }

    #[cfg(any(test, feature = "test-support"))]
    pub fn of(
        current_size: u8,
        total_copies_per_id: [u8; MAX_UNIQUE_CARDS_IN_DECK],
        revealed_copies_per_index: [u8; MAX_UNIQUE_CARDS_IN_DECK],
        empathy_by_index: [CardIdentityMask; MAX_CARDS_IN_DECK],
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

    /// Decrement the size of this deck by the given amount
    pub fn decrement_size(&mut self, amount: u8) {
        self.current_size -= amount;
    }

    #[must_use]
    pub fn get_global_empathy(&self, deck_card_index: CardDeckIndex) -> CardIdentityMask {
        self.empathy_by_index[deck_card_index as usize]
    }

    pub fn reveal_card(
        &mut self,
        card_position_in_starting_deck: CardDeckIndex,
        card_id: VariantCardId,
    ) {
        let mut worklist: SmallVec<[(CardDeckIndex, VariantCardId); 4]> =
            smallvec::smallvec![(card_position_in_starting_deck, card_id)];
        while let Some((pos, id)) = worklist.pop() {
            let identity_mask: VariantCardsBitField = 1 << id;
            self.revealed_copies_per_index[id] += 1;
            self.empathy_by_index[pos as usize] = CardIdentityMask::known(id);
            self.revealed_indexes |= 1u64 << pos;
            if self.revealed_copies_per_index[id] == self.total_copies_per_id[id] {
                let valid_mask = (1u64 << self.current_size) - 1;
                let mut unrevealed = !self.revealed_indexes & valid_mask;
                while unrevealed != 0 {
                    let index = unrevealed.trailing_zeros() as usize;
                    unrevealed &= unrevealed - 1;
                    let old_empathy = self.empathy_by_index[index];
                    if let Some(new_empathy) = old_empathy.exclude(identity_mask) {
                        if new_empathy.is_exactly_known() && !old_empathy.is_exactly_known() {
                            worklist.push((
                                index as CardDeckIndex,
                                new_empathy.known_card_id().unwrap(),
                            ));
                        }
                        self.empathy_by_index[index] = new_empathy;
                    }
                }
            }
        }
    }

    pub fn update_positive_empathy(
        &mut self,
        card_position_in_starting_deck: CardDeckIndex,
        empathy_update: VariantCardsBitField,
    ) {
        let index = card_position_in_starting_deck as usize;
        let current_empathy = self.empathy_by_index[index];
        if let Some(new_empathy) = current_empathy.narrow(empathy_update) {
            self.empathy_by_index[index] = new_empathy;
            if new_empathy.is_exactly_known() {
                let revealed_card_id = new_empathy.known_card_id().unwrap();
                self.reveal_card(card_position_in_starting_deck, revealed_card_id);
            }
        }
    }

    pub fn update_negative_empathy(
        &mut self,
        card_position_in_starting_deck: CardDeckIndex,
        empathy_update: VariantCardsBitField,
    ) {
        let index = card_position_in_starting_deck as usize;
        let current_empathy = self.empathy_by_index[index];
        if let Some(new_empathy) = current_empathy.exclude(empathy_update) {
            self.empathy_by_index[index] = new_empathy;
            if new_empathy.is_exactly_known() {
                let revealed_card_id = new_empathy.known_card_id().unwrap();
                self.reveal_card(card_position_in_starting_deck, revealed_card_id);
            }
        }
    }
}

#[cfg(any(test, feature = "test-support"))]
pub mod unit_test_constants {
    use crate::game::card::CardIdentityMask;
    use crate::game::deck::Deck;
    use crate::game::deck::unit_test_constants::novariant_constants::COPIES_COUNT_BY_ID;
    use crate::game::variant::test_variants::NO_VARIANT;
    use crate::game::{MAX_CARDS_IN_DECK, MAX_UNIQUE_CARDS_IN_DECK};

    pub mod novariant_constants {
        use crate::game::MAX_UNIQUE_CARDS_IN_DECK;
        use crate::game::card::{VariantCardId, VariantCardsBitField};

        pub const COPIES_COUNT_BY_ID: [u8; MAX_UNIQUE_CARDS_IN_DECK] = [
            3, 2, 2, 2, 1, 3, 2, 2, 2, 1, 3, 2, 2, 2, 1, 3, 2, 2, 2, 1, 3, 2, 2, 2, 1, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
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
        pub const ALL_CARDS_MASK: VariantCardsBitField = (1 << 25) - 1;
    }

    pub const NEW_DECK: Deck = Deck {
        current_size: 50,
        total_copies_per_id: COPIES_COUNT_BY_ID,
        revealed_copies_per_index: [0; MAX_UNIQUE_CARDS_IN_DECK],
        empathy_by_index: [CardIdentityMask::all(&NO_VARIANT); MAX_CARDS_IN_DECK],
        revealed_indexes: 0,
    };
}

#[cfg(test)]
mod tests {
    use super::unit_test_constants::NEW_DECK;
    use super::unit_test_constants::novariant_constants::*;
    use super::*;
    use crate::game::variant::test_variants::NO_VARIANT;

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
        let expected = CardIdentityMask::from_bits(R3_MASK);
        let actual = deck.empathy_by_index[42];
        assert_eq!(expected, actual);
    }

    #[test]
    fn should_update_empathy_indirectly_when_all_copies_are_revealed() {
        let mut deck = NEW_DECK.clone();

        deck.reveal_card(42, 2);
        deck.reveal_card(22, 2);
        assert_eq!(
            CardIdentityMask::from_bits(R3_MASK),
            deck.empathy_by_index[42]
        );
        assert_eq!(
            CardIdentityMask::from_bits(R3_MASK),
            deck.empathy_by_index[22]
        );
        assert_eq!(
            CardIdentityMask::from_bits(ALL_CARDS_MASK & !R3_MASK),
            deck.empathy_by_index[1]
        );
    }

    #[test]
    fn should_recursively_update_empathy() {
        let empathy_by_index: [CardIdentityMask; MAX_CARDS_IN_DECK] = {
            let mut arr = [CardIdentityMask::all(&NO_VARIANT); MAX_CARDS_IN_DECK];
            let pairs: &[(usize, VariantCardsBitField)] = &[
                (0, R1_MASK),
                (1, R1_MASK),
                (2, R1_MASK),
                (3, R2_MASK),
                (4, R2_MASK),
                (5, R3_MASK),
                (6, R3_MASK),
                (7, R4_MASK),
                (8, R4_MASK),
                (9, R5_MASK),
                (10, Y1_MASK),
                (11, Y1_MASK),
                (12, Y1_MASK),
                (13, Y2_MASK),
                (14, Y2_MASK),
                (15, Y3_MASK),
                (16, Y4_MASK),
                (17, Y4_MASK),
                (18, G1_MASK),
                (19, G1_MASK),
                (20, G1_MASK),
                (21, G2_MASK),
                (22, G2_MASK),
                (23, G3_MASK),
                (24, G3_MASK),
                (25, G4_MASK),
                (26, G4_MASK),
                (27, G5_MASK),
                (28, B1_MASK),
                (29, B1_MASK),
                (30, B1_MASK),
                (31, B2_MASK),
                (32, B2_MASK),
                (33, B3_MASK),
                (34, B3_MASK),
                (35, B4_MASK),
                (36, B4_MASK),
                (37, B5_MASK),
                (38, P1_MASK),
                (39, P1_MASK),
                (40, P1_MASK),
                (41, P2_MASK),
                (42, P2_MASK),
                (43, P3_MASK),
                (44, P3_MASK),
                (45, P4_MASK),
                (46, P4_MASK),
                (47, Y3_MASK | Y5_MASK),
                (48, Y3_MASK | Y5_MASK),
                (49, Y5_MASK | P5_MASK),
            ];
            for &(i, m) in pairs {
                arr[i] = CardIdentityMask::from_bits(m);
            }
            arr
        };
        let revealed_copies_per_index: [u8; MAX_UNIQUE_CARDS_IN_DECK] = [
            3, 2, 2, 2, 1, 3, 2, 1, 2, 0, 3, 2, 2, 2, 1, 3, 2, 2, 2, 1, 3, 2, 2, 2, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ];
        let mut deck = Deck::of(
            50,
            COPIES_COUNT_BY_ID,
            revealed_copies_per_index,
            empathy_by_index,
            0,
        );

        deck.reveal_card(47, 7);
        assert_eq!(
            CardIdentityMask::from_bits(Y3_MASK),
            deck.empathy_by_index[47]
        );
        assert_eq!(
            CardIdentityMask::from_bits(Y5_MASK),
            deck.empathy_by_index[48]
        );
        assert_eq!(
            CardIdentityMask::from_bits(P5_MASK),
            deck.empathy_by_index[49]
        );
    }
}
