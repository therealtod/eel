use crate::game::card::{VariantCardId, VariantCardsBitField};
use crate::game::clue::Clue;
use crate::game::{MAX_CLUE_TYPES, MAX_CLUE_VALUES_PER_TYPE, MAX_UNIQUE_CARDS_IN_DECK};

/// Data which is specific of the variant selected for the game
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Variant {
    /// Amount of half clue tokens are awarded to the team when a card is discarded
    pub bonus_half_clue_tokens_for_discard: u8,
    /// Amount of half clue tokens awarded to the team when the last card of a stack gets played
    /// successfully
    pub bonus_half_clue_tokens_for_completing_stack: u8,
    /// For each index `i` of the array, the value is equal to the number of copies of the given 
    /// card with id equal to `i`
    pub card_copies_count_by_id: [u8; MAX_UNIQUE_CARDS_IN_DECK],
    /// Total number of cards (counting all the copies) that compose the deck used in the game
    pub deck_size: u8,
    /// For each pair of indexes `i` and `j` of the array, the value is a [VariantCardsBitField] 
    /// bitfield of the cards which are touched by a clue type of type `i` with value `j`
    pub empathy_by_clue: [[VariantCardsBitField; MAX_CLUE_VALUES_PER_TYPE]; MAX_CLUE_TYPES],
    /// The maximum size of each playing stack 
    pub stacks_size: u8,
    /// How many separate suits are included in the deck
    pub number_of_suits: u8,
}

/// Clue type index for rank clues.
pub const RANK_CLUE_TYPE: usize = 1;

impl Variant {
    pub fn get_card_suit_index(&self, variant_card_id: VariantCardId) -> usize {
        variant_card_id / self.stacks_size as usize
    }

    /// Returns the empathy mask for the given clue.
    ///
    /// Rank clue values are **1-based** (pass `5` for a rank-5 clue).
    /// Color clue values are **0-based** suit indices (pass `0` for Red, `1` for Yellow, …).
    pub fn empathy_for_clue(&self, clue: &Clue) -> VariantCardsBitField {
        let index = if clue.clue_type as usize == RANK_CLUE_TYPE {
            (clue.clue_value - 1) as usize
        } else {
            clue.clue_value as usize
        };
        self.empathy_by_clue[clue.clue_type as usize][index]
    }
}

pub mod test_variants {
    use crate::game::card::VariantCardsBitField;
    use crate::game::variant::Variant;
    use crate::game::{MAX_CLUE_TYPES, MAX_CLUE_VALUES_PER_TYPE, MAX_UNIQUE_CARDS_IN_DECK};

    const NO_VARIANT_CARD_COPIES_COUNT_BY_ID: [u8; MAX_UNIQUE_CARDS_IN_DECK] = [
        3, 2, 2, 2, 1, 3, 2, 2, 2, 1, 3, 2, 2, 2, 1, 3, 2, 2, 2, 1, 3, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
    ];

    // Color clue value i touches suit i: bits [i*5 .. i*5+4]
    // Rank clue value r touches rank r in every suit: bits r, r+5, r+10, r+15, r+20
    const NO_VARIANT_EMPATHY_BY_CLUE: [[VariantCardsBitField; MAX_CLUE_VALUES_PER_TYPE]; MAX_CLUE_TYPES] = [
        [
            0b11111 << 0,   // Red:    R1..R5  (bits  0-4)
            0b11111 << 5,   // Yellow: Y1..Y5  (bits  5-9)
            0b11111 << 10,  // Green:  G1..G5  (bits 10-14)
            0b11111 << 15,  // Blue:   B1..B5  (bits 15-19)
            0b11111 << 20,  // Purple: P1..P5  (bits 20-24)
            0b0,
        ],
        [
            // "1" clue: R1(0), Y1(5), G1(10), B1(15), P1(20)
            1 | 1 << 5 | 1 << 10 | 1 << 15 | 1 << 20,
            // "2" clue: R2(1), Y2(6), G2(11), B2(16), P2(21)
            1 << 1 | 1 << 6 | 1 << 11 | 1 << 16 | 1 << 21,
            // "3" clue: R3(2), Y3(7), G3(12), B3(17), P3(22)
            1 << 2 | 1 << 7 | 1 << 12 | 1 << 17 | 1 << 22,
            // "4" clue: R4(3), Y4(8), G4(13), B4(18), P4(23)
            1 << 3 | 1 << 8 | 1 << 13 | 1 << 18 | 1 << 23,
            // "5" clue: R5(4), Y5(9), G5(14), B5(19), P5(24)
            1 << 4 | 1 << 9 | 1 << 14 | 1 << 19 | 1 << 24,
            0b0,
        ],
    ];
    
    pub const NO_VARIANT: Variant = Variant {
        bonus_half_clue_tokens_for_discard: 2,
        bonus_half_clue_tokens_for_completing_stack: 2,
        card_copies_count_by_id: NO_VARIANT_CARD_COPIES_COUNT_BY_ID,
        deck_size: 50,
        empathy_by_clue: NO_VARIANT_EMPATHY_BY_CLUE,
        number_of_suits: 5,
        stacks_size: 5,
    };
}
