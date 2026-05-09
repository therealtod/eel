pub mod card_identity_mask;
pub mod copies_counting_card_collection;

pub use card_identity_mask::CardIdentityMask;

pub type CardDeckIndex = u8;
pub type VariantCardsBitField = u64;
pub type DeckCardsBitField = u64;
pub type VariantCardId = usize;
