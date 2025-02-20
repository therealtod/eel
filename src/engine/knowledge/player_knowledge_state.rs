use smallvec::SmallVec;

use crate::engine::knowledge::knowledge_update::KnowledgeUpdate;
use crate::engine::signal::Signal;
use crate::game::card::{CardDeckIndex, DeckCardsBitField, VariantCardId, VariantCardsBitField};
use crate::game::MAX_CARDS_IN_DECK;

/// Per-player mutable knowledge storage.
///
/// Tracks what a specific player knows (or can infer) about every card in the deck.
/// One instance per player, stored inside [`TeamKnowledge`](super::team_knowledge::TeamKnowledge).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerKnowledgeState {
    /// Index of the player this knowledge belongs to.
    pub player_index: usize,
    /// For each card (indexed by deck position): which variant-card identities are still possible.
    pub empathy: [VariantCardsBitField; MAX_CARDS_IN_DECK],
    /// For each card: identities inferred through convention interpretation (subset of empathy).
    pub inferred_identities: [VariantCardsBitField; MAX_CARDS_IN_DECK],
    /// Signals (play/discard/save) attached to cards by convention interpretation.
    /// `SmallVec<[Signal; 2]>` avoids heap allocation for the common 0–2 signals case.
    pub signals: [SmallVec<[Signal; 2]>; MAX_CARDS_IN_DECK],
    /// Bitfield of cards whose identity is visible to this player.
    pub visible_cards: DeckCardsBitField,
    /// Bitfield of cards currently in this player's own hand.
    pub own_hand: DeckCardsBitField,
}

impl PlayerKnowledgeState {
    /// Create a new `PlayerKnowledgeState` with no knowledge.
    pub fn new(player_index: usize) -> Self {
        PlayerKnowledgeState {
            player_index,
            empathy: [VariantCardsBitField::MAX; MAX_CARDS_IN_DECK],
            inferred_identities: [0; MAX_CARDS_IN_DECK],
            signals: std::array::from_fn(|_| SmallVec::new()),
            visible_cards: 0,
            own_hand: 0,
        }
    }

    /// Create a default (empty) state, used for padding fixed-size arrays.
    pub fn empty() -> Self {
        Self::new(0)
    }

    /// Mark a card as revealed (visible) and set its identity.
    pub fn update_with_revealed_card(
        &mut self,
        card_deck_index: CardDeckIndex,
        card_id: VariantCardId,
    ) {
        let idx = card_deck_index as usize;
        self.empathy[idx] = 1 << card_id;
        self.visible_cards |= 1 << card_deck_index;
    }

    /// Restrict the possible identities of a card to only those in the given mask.
    pub fn narrow_possibilities(
        &mut self,
        card_deck_index: CardDeckIndex,
        mask: VariantCardsBitField,
    ) {
        self.empathy[card_deck_index as usize] &= mask;
    }

    /// Attach a signal to a card.
    pub fn add_signal(
        &mut self,
        card_deck_index: CardDeckIndex,
        signal: Signal,
    ) {
        self.signals[card_deck_index as usize].push(signal);
    }

    /// Apply a batch of [`KnowledgeUpdate`]s produced by convention interpretation.
    pub fn apply_updates(&mut self, updates: &[KnowledgeUpdate]) {
        for update in updates {
            match update {
                KnowledgeUpdate::NarrowPossibilities { card_deck_index, mask } => {
                    self.narrow_possibilities(*card_deck_index, *mask);
                }
                KnowledgeUpdate::AddSignal { card_deck_index, signal } => {
                    self.add_signal(*card_deck_index, signal.clone());
                }
            }
        }
    }

    /// Get the possible identities for a card.
    pub fn possible_identities(&self, card_deck_index: CardDeckIndex) -> VariantCardsBitField {
        self.empathy[card_deck_index as usize]
    }
}
