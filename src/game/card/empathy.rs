use super::{VariantCardId, VariantCardsBitField};

/// A non-zero bitmask of possible card identities for a single deck position.
///
/// Invariant: the inner bitmask is never zero. A zero mask would mean a card has no possible
/// identity — always a contradiction in a valid game state.
///
/// Methods that could produce an empty result (like [`narrow`] and [`exclude`]) return
/// `None` so callers decide whether to keep the old value or treat the situation as a bug.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Empathy(VariantCardsBitField);

impl Empathy {
    /// All identities are possible for a variant with the given card mask.
    pub const fn all(all_cards_mask: VariantCardsBitField) -> Self {
        Empathy(all_cards_mask)
    }

    /// Exactly one identity is possible.
    pub const fn known(card_id: VariantCardId) -> Self {
        Empathy(1 << card_id)
    }

    /// Construct from a raw bitmask. Returns `None` if `bits` is zero.
    pub fn from_bits(bits: VariantCardsBitField) -> Option<Self> {
        if bits == 0 { None } else { Some(Empathy(bits)) }
    }

    /// Returns `true` if exactly one identity remains possible.
    pub fn is_exactly_known(self) -> bool {
        self.0.is_power_of_two()
    }

    /// Returns the single possible identity if exactly one remains, otherwise `None`.
    pub fn known_card_id(self) -> Option<VariantCardId> {
        if self.is_exactly_known() {
            Some(self.0.trailing_zeros() as VariantCardId)
        } else {
            None
        }
    }

    /// Number of possible identities.
    pub fn count_possibilities(self) -> u32 {
        self.0.count_ones()
    }

    /// Raw bitmask, for operations that need to intersect with other bitmasks.
    pub fn as_bits(self) -> VariantCardsBitField {
        self.0
    }

    /// Keep only identities also present in `mask` (positive clue narrowing).
    /// Returns `None` if the intersection would be empty.
    pub fn narrow(self, mask: VariantCardsBitField) -> Option<Self> {
        let bits = self.0 & mask;
        if bits == 0 { None } else { Some(Empathy(bits)) }
    }

    /// Remove identities present in `mask` (negative clue elimination).
    /// Returns `None` if all identities would be eliminated.
    pub fn exclude(self, mask: VariantCardsBitField) -> Option<Self> {
        let bits = self.0 & !mask;
        if bits == 0 { None } else { Some(Empathy(bits)) }
    }
}
