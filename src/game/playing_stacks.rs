use crate::game::card::{CardPositionInStartingDeck, UniqueCardId};
use crate::game::card::card_collection::CardCollection;
use crate::game::deck::Deck;
use crate::game::card::hanabi_card::HanabiCard;
use crate::game::MAX_STACKS_IN_GAME;
use crate::game::rank::Rank;
use crate::game::suit::SuitId;
use crate::game::variant::Variant;

#[derive(Debug)]
pub struct PlayingStacks {
    cards: CardCollection,
}

impl PlayingStacks {
    pub fn play(&mut self, card_id: UniqueCardId, variant: &Variant) -> (bool, u8) {
        if self.get_next_playable_cards(variant).contains(&card_id) {
            self.cards.add_card(card_id);
            (true, Self::gives_bonus_token(card_id, variant))
        } else {
            (false, 0)
        }
    }

    pub fn get_next_playable_cards(&self, variant: &Variant) -> Vec<UniqueCardId> {
        todo!()
    }

    fn gives_bonus_token(card_id: UniqueCardId, variant: &Variant) -> u8 {
        let card_identity = variant.get_card_identity(card_id);
        if card_identity.rank == Rank::Five { // TODO: generalize
            2
        } else {
            0
        }
    }
}
