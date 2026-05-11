use super::{VariantCardId, VariantCardsBitField};
use crate::game::variant::Variant;

/// A bitmask of possible card identities for a single deck position.
///
/// Methods that could produce an empty result (like [`narrow`] and [`exclude`]) return
/// `None` so callers decide whether to keep the old value or treat the situation as a bug.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct CardIdentityMask(VariantCardsBitField);

impl CardIdentityMask {
    /// All identities are possible for a variant with the given card mask.
    #[must_use]
    pub const fn all(variant: &Variant) -> Self {
        CardIdentityMask(variant.all_cards_mask())
    }

    /// Exactly one identity is possible.
    #[must_use]
    pub const fn known(card_id: VariantCardId) -> Self {
        CardIdentityMask(1 << card_id)
    }

    /// Construct from a raw bitmask. Returns `None` if `bits` is zero.
    #[must_use]
    pub fn from_bits(bits: VariantCardsBitField) -> Self {
        CardIdentityMask(bits)
    }

    /// Returns `true` if exactly one identity remains possible.
    #[must_use]
    pub fn is_exactly_known(self) -> bool {
        self.0.is_power_of_two()
    }

    /// Returns the single possible identity if exactly one remains, otherwise `None`.
    #[must_use]
    pub fn known_card_id(self) -> Option<VariantCardId> {
        if self.is_exactly_known() {
            Some(self.0.trailing_zeros() as VariantCardId)
        } else {
            None
        }
    }

    /// Number of possible identities.
    #[must_use]
    pub fn count_possibilities(self) -> u32 {
        self.0.count_ones()
    }

    /// Raw bitmask, for operations that need to intersect with other bitmasks.
    #[must_use]
    pub fn as_bits(self) -> VariantCardsBitField {
        self.0
    }

    /// Keep only identities also present in `mask` (positive clue narrowing).
    /// Returns `None` if the intersection would be empty.
    #[must_use]
    pub fn narrow(self, mask: VariantCardsBitField) -> Option<Self> {
        let bits = self.0 & mask;
        if bits == 0 {
            None
        } else {
            Some(CardIdentityMask(bits))
        }
    }

    /// Remove identities present in `mask` (negative clue elimination).
    /// Returns `None` if all identities would be eliminated.
    #[must_use]
    pub fn exclude(self, mask: VariantCardsBitField) -> Option<Self> {
        let bits = self.0 & !mask;
        if bits == 0 {
            None
        } else {
            Some(CardIdentityMask(bits))
        }
    }

    #[must_use]
    pub fn contains(self, variant_card_id: VariantCardId) -> bool {
        self.0 & 1 << variant_card_id != 0
    }
}
