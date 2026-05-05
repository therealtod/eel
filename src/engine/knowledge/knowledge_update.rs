use crate::engine::convention::hgroup::signal::Signal;
use crate::game::card::{CardDeckIndex, VariantCardsBitField};

/// Describes a discrete update to a player's knowledge state,
/// produced by convention interpretation.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum KnowledgeUpdate {
    /// Restrict the possible identities of a card to only those in the given mask.
    NarrowPossibilities {
        card_deck_index: CardDeckIndex,
        mask: VariantCardsBitField,
    },
    /// Attach a signal (play, discard, save) to a card.
    AddSignal {
        card_deck_index: CardDeckIndex,
        signal: Signal,
    },
}
