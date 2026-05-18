use crate::game::SlotIndex;
use crate::game::card::{CardDeckIndex, VariantCardId};

/// Convention-generated annotations attached to specific cards in a player's hand.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Signal {
    /// The holder is committed to playing this card.
    ///
    /// `committed_identity` is the identity the convention says this card has — the holder
    /// can use it to resolve their own empathy.
    Play {
        card_deck_index: CardDeckIndex,
        committed_identity: VariantCardId,
    },
    Discard {
        slot_index: SlotIndex,
        turn: usize,
    },
    Save {
        slot_index: SlotIndex,
        turn: usize,
    },
}
