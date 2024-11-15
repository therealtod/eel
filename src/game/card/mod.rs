pub mod copies_counting_card_collection;
pub mod empathy;

pub use empathy::Empathy;

pub type CardDeckIndex = u8;
pub type VariantCardsBitField = u64;
pub type DeckCardsBitField = u64;
pub type VariantCardId = usize;
