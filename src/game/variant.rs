use crate::game::card::hanabi_card::HanabiCard;
use crate::game::card::UniqueCardId;

#[derive(Debug)]
pub struct Variant {
}

impl Variant {
    pub fn get_card_identity(&self, card_id: UniqueCardId) -> HanabiCard {
        todo!()
    }

    pub fn get_bonus_half_tokens_for_discarding(&self) -> u8 { // TODO: generalize
        2
    }
}