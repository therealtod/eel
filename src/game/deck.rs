use crate::game::card::{CardPositionInStartingDeck, Empathy, UniqueCardId};
use crate::game::card::hanabi_card::HanabiCard;
use crate::game::{MAX_CARDS_IN_DECK, MAX_UNIQUE_CARDS_IN_DECK};

#[derive(PartialEq, Debug)]
pub struct Deck {
    pub current_size: usize,
    total_copies_per_id: [u8; MAX_UNIQUE_CARDS_IN_DECK],
    revealed_copies_per_index: [u8; MAX_UNIQUE_CARDS_IN_DECK],
    revealed_values: [UniqueCardId; MAX_CARDS_IN_DECK],
    empathy_by_index: [Empathy; MAX_CARDS_IN_DECK],
    revealed_indexes: u64,
}

impl Deck {
    pub fn new() -> Self {
        todo!()
    }

    pub fn draw(&mut self) {
        self.current_size -= 1;
    }
    pub fn get_global_empathy(&self, deck_card_index: CardPositionInStartingDeck) -> Empathy {
        self.empathy_by_index[deck_card_index]
    }
    pub fn reveal_card(
        &mut self,
        card_position_in_starting_deck: CardPositionInStartingDeck,
        card_id: UniqueCardId,
    ) {
        let card_empathy = 1 << card_id;
        self.revealed_values[card_position_in_starting_deck] = card_id;
        self.revealed_copies_per_index[card_id] += 1;
        self.empathy_by_index[card_position_in_starting_deck] = card_empathy;
        self.revealed_indexes |= 1<< card_position_in_starting_deck;
        if self.revealed_copies_per_index[card_id] == self.total_copies_per_id[card_id] {
            for (index, empathy)
            in &mut self.empathy_by_index.iter_mut().enumerate() {
                if self.revealed_indexes & (1 << index) == 0 {
                    let new_empathy = *empathy & !card_empathy;
                    *empathy = new_empathy;
                }
            }
        }
    }

    pub fn update_positive_empathy(
        &mut self,
        card_position_in_starting_deck: &CardPositionInStartingDeck,
        empathy_update: Empathy
    ) {
        self.empathy_by_index[*card_position_in_starting_deck] &= empathy_update;
        let new_empathy = self.empathy_by_index[*card_position_in_starting_deck];
        if new_empathy.count_ones() == 1 {
            let revealed_card_id: UniqueCardId = new_empathy.trailing_zeros() as UniqueCardId;
            self.reveal_card(*card_position_in_starting_deck, revealed_card_id);
        }
    }

    pub fn update_negative_empathy(
        &mut self,
        card_position_in_starting_deck: &CardPositionInStartingDeck,
        empathy_update: Empathy
    ) {
        self.empathy_by_index[*card_position_in_starting_deck] &= empathy_update;
        let new_empathy = self.empathy_by_index[*card_position_in_starting_deck];
        if new_empathy.count_ones() == 1 {
            let revealed_card_id: UniqueCardId = new_empathy.trailing_zeros() as UniqueCardId;
            self.reveal_card(*card_position_in_starting_deck, revealed_card_id);
        }
    }

    fn reveal_cards(
        &mut self,
        revealed_deck_index_card_id_pairs: &[(CardPositionInStartingDeck, UniqueCardId)],
    ) {
        let mut revealed_cards_mask = 0;
        for &(card_position_in_starting_deck, card_id)
        in revealed_deck_index_card_id_pairs {
            self.revealed_indexes |= 1 << card_position_in_starting_deck;
            self.revealed_copies_per_index[card_id] += 1;
            revealed_cards_mask |= 1 << card_id;
        }

        let mut tmp_mask = revealed_cards_mask;
        while tmp_mask != 0 {
            let revealed_card_id: UniqueCardId = tmp_mask.trailing_zeros();
            tmp_mask ^= 1<< revealed_card_id;
            if self.revealed_copies_per_index[revealed_card_id]
                == self.total_copies_per_id[revealed_card_id] {
                for (index, empathy)
                in &mut self.empathy_by_index.iter_mut().enumerate() {
                    if self.revealed_indexes & (1 << index) == 0 {
                        let new_empathy = *empathy & !(1<< revealed_card_id);
                        *empathy = new_empathy;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::game::ALL_CARDS_MASK;
    use super::*;
    const COPIES_COUNT_BY_ID: [u8; MAX_UNIQUE_CARDS_IN_DECK] = [
3,2,2,2,1,3,2,2,2,1,3,2,2,2,1,3,2,2,2,1,3,2,2,2,1,0,0,0,0,0,0,0,0,0,0
    ];
    const R1_MASK: Empathy = 1;
    const R2_MASK: Empathy = 2;
    const R3_MASK: Empathy = 1<<2;
    const R4_MASK: Empathy = 1<<3;
    const R5_MASK: Empathy = 1<<4;
    const Y1_MASK: Empathy = 1<<5;
    const Y2_MASK: Empathy = 1<<6;
    const Y3_MASK: Empathy = 1<<7;
    const Y4_MASK: Empathy = 1<<8;
    const Y5_MASK: Empathy = 1<<9;
    const G1_MASK: Empathy = 1<<10;
    const G2_MASK: Empathy = 1<<11;
    const G3_MASK: Empathy = 1<<12;
    const G4_MASK: Empathy = 1<<13;
    const G5_MASK: Empathy = 1<<14;
    const B1_MASK: Empathy = 1<<15;
    const B2_MASK: Empathy = 1<<16;
    const B3_MASK: Empathy = 1<<17;
    const B4_MASK: Empathy = 1<<18;
    const B5_MASK: Empathy = 1<<19;
    const P1_MASK: Empathy = 1<<20;
    const P2_MASK: Empathy = 1<<21;
    const P3_MASK: Empathy = 1<<22;
    const P4_MASK: Empathy = 1<<23;
    const P5_MASK: Empathy = 1<<24;


    #[test]
    fn should_update_empathy_of_revealed_index() {
        let mut deck = Deck{
            current_size: 50,
            total_copies_per_id: COPIES_COUNT_BY_ID,
            revealed_copies_per_index: [0; MAX_UNIQUE_CARDS_IN_DECK],
            revealed_values: [0; MAX_CARDS_IN_DECK],
            empathy_by_index: [ALL_CARDS_MASK; MAX_CARDS_IN_DECK],
            revealed_indexes: 0,
        };

        deck.reveal_card(42, 2);
        let expected = R3_MASK;
        let actual = deck.empathy_by_index[42];
        assert_eq!(expected, actual);
    }

    #[test]
    fn should_update_empathy_indirectly_when_all_copies_are_revealed() {
        let mut deck = Deck{
            current_size: 50,
            total_copies_per_id: COPIES_COUNT_BY_ID,
            revealed_copies_per_index: [0; MAX_UNIQUE_CARDS_IN_DECK],
            revealed_values: [0; MAX_CARDS_IN_DECK],
            empathy_by_index: [ALL_CARDS_MASK; MAX_CARDS_IN_DECK],
            revealed_indexes: 0,
        };

        deck.reveal_card(42, 2);
        deck.reveal_card(22, 2);
        assert_eq!(R3_MASK, deck.empathy_by_index[42]);
        assert_eq!(R3_MASK, deck.empathy_by_index[22]);
        assert_eq!(Empathy::MAX & !R3_MASK, deck.empathy_by_index[1]);
    }

    #[test]
    fn should_recursively_update_empathy() {
        let mut deck = Deck{
            current_size: 50,
            total_copies_per_id: COPIES_COUNT_BY_ID,
            revealed_copies_per_index: [
                3,2,2,2,1,3,2,1,2,0,3,2,2,2,1,3,2,2,2,1,3,2,2,2,0,0,0,0,0,0,0,0,0,0,0
            ],
            revealed_values: [0; MAX_CARDS_IN_DECK],
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
