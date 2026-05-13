use crate::engine::convention::hgroup::signal::Signal;
use crate::game::card::{CardDeckIndex, VariantCardsBitField};
use crate::game::state::PlayerIndex;

/// Identifier of a tracked hypothesis. Assigned by the dispatcher when storing a
/// hypothesis in a player's knowledge.
pub type HypothesisId = u32;

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

impl KnowledgeUpdate {
    /// Returns the deck index targeted by this update.
    #[must_use]
    pub fn card_deck_index(&self) -> CardDeckIndex {
        match self {
            KnowledgeUpdate::NarrowPossibilities {
                card_deck_index, ..
            }
            | KnowledgeUpdate::AddSignal {
                card_deck_index, ..
            } => *card_deck_index,
        }
    }
}

/// One tech's interpretation of an observed action, from a single observer's POV.
///
/// A hypothesis is a *single* interpretation. It carries the narrowings/signals it
/// claims, plus an optional [`PendingTrigger`] that resolves the hypothesis into
/// either confirmation (the hypothesis becomes the only valid interpretation) or
/// rejection (the hypothesis is dropped).
///
/// Multiple hypotheses can coexist within the same *cohort* (all hypotheses from a
/// single observed action). The dispatcher composes a cohort by collecting one
/// hypothesis per matching tech — the receiver's effective narrowing for any card
/// is the **union** of cohort hypothesis masks targeting that card, intersected
/// with prior baseline knowledge.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Hypothesis {
    /// The narrowings and signals this hypothesis claims.
    pub immediate: Vec<KnowledgeUpdate>,
    /// If `Some`, the hypothesis is provisional: it survives only if the trigger
    /// resolves with confirmation. On rejection it is dropped from the cohort.
    pub trigger: Option<PendingTrigger>,
}

impl Hypothesis {
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn unconditional(immediate: Vec<KnowledgeUpdate>) -> Self {
        Self {
            immediate,
            trigger: None,
        }
    }

    #[must_use]
    pub fn provisional(immediate: Vec<KnowledgeUpdate>, trigger: PendingTrigger) -> Self {
        Self {
            immediate,
            trigger: Some(trigger),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.immediate.is_empty() && self.trigger.is_none()
    }
}

/// A hypothesis that has been stored in a player's knowledge, with its dispatcher-
/// assigned ids.
///
/// `cohort_id` is shared across all hypotheses produced by interpreting the *same*
/// observed action. When one hypothesis in a cohort confirms, its siblings are
/// pruned. Rejection drops only the rejected hypothesis.
///
/// `tier` identifies the hypothesis's priority rank within its cohort:
/// - `0` = primary (highest-priority matching tier; active immediately).
/// - `1` = fallback (next-highest matching tier; dormant until all tier-0 siblings
///   in the cohort are rejected, at which point they are promoted to tier 0).
///
/// Only tier-0 hypotheses contribute to `effective_inferred_mask` and `has_play_signal`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrackedHypothesis {
    pub id: HypothesisId,
    pub cohort_id: HypothesisId,
    /// Priority tier within the cohort. `0` = active primary, `1` = dormant fallback.
    pub tier: u8,
    pub immediate: Vec<KnowledgeUpdate>,
    pub trigger: Option<PendingTrigger>,
}

/// Triggers that resolve a hypothesis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PendingTrigger {
    /// Resolves on `player`'s next action.
    /// - **Confirm** when that action is `Play { card_deck_index: expected_card, .. }`.
    /// - **Reject** for any other action by `player`.
    ///
    /// `deadline_turn` records the turn after which the hypothesis is forced to
    /// resolve (used as a backstop when the player is somehow skipped).
    BlindPlay {
        player: PlayerIndex,
        expected_card: CardDeckIndex,
        deadline_turn: usize,
    },
}
