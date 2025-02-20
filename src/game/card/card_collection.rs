use crate::game::card::{CardDeckIndex, VariantCardId};

/// An unordered collection of [HanabiCard]
pub trait InGameCardCollection {
    /// Return an empty [InGameCardCollection]
    fn empty() -> Self;

    /// Increase the size of the collection
    fn add_card(&mut self);

    /// Add the card with known identity `card_id` to this collection
    fn add_card_with_id(&mut self, card_id: VariantCardId);

    /// Return [true] if this collection contains the card with the given [card_id]
    fn contains_card_with_id(&self, card_id: VariantCardId) -> bool;
    
    /// Return the size of this collection
    fn size(&self) -> u8;
}
