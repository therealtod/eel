use smallvec::SmallVec;

use crate::engine::knowledge::knowledge_update::KnowledgeUpdate;
use crate::engine::convention::hgroup::signal::Signal;
use crate::game::MAX_CARDS_IN_DECK;
use crate::game::card::{CardDeckIndex, DeckCardsBitField, Empathy, VariantCardId, VariantCardsBitField};
use crate::game::state::table_state::TableState;
use crate::game::variant::Variant;

/// Per-player mutable knowledge storage.
///
/// Tracks what a specific player knows (or can infer) about every card in the deck.
/// One instance per player, stored inside [`TeamKnowledge`](super::team_knowledge::TeamKnowledge).
///
/// # Knowledge sources
///
/// - **Game-rule empathy**: Computed on-demand from game state (positive clues,
///   negative clues, discard pile, playing stacks, visible cards in other players' hands).
/// - **Inferred identities**: Convention interpretation results from tech `knowledge_updates`.
/// - **Signals**: Convention signals attached to cards (play/discard/save).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerKnowledgeState {
    /// Index of the player this knowledge belongs to.
    pub player_index: usize,
    /// For each card: identities inferred through convention interpretation.
    /// This is the subset of game-rule empathy that results from decoding conventions.
    pub inferred_identities: [Option<Empathy>; MAX_CARDS_IN_DECK],
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
            inferred_identities: [None; MAX_CARDS_IN_DECK],
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
        self.inferred_identities[idx] = Some(Empathy::known(card_id));
        self.visible_cards |= 1 << card_deck_index;
    }

    /// Restrict the possible identities of a card to only those in the given mask.
    /// Updates inferred_identities (convention-based narrowing).
    /// No-ops if the intersection would be empty (avoids corrupting known identities).
    pub fn narrow_inferred(
        &mut self,
        card_deck_index: CardDeckIndex,
        mask: VariantCardsBitField,
        variant: &Variant
    ) {
        let idx = card_deck_index as usize;
        let current = self.inferred_identities[idx].unwrap_or_else(|| Empathy::all(variant));
        if let Some(new_empathy) = current.narrow(mask) {
            self.inferred_identities[idx] = Some(new_empathy);
            if new_empathy.is_exactly_known() {
                self.visible_cards |= 1 << card_deck_index;
            }
        }
    }

    /// Attach a signal to a card.
    pub fn add_signal(&mut self, card_deck_index: CardDeckIndex, signal: Signal) {
        self.signals[card_deck_index as usize].push(signal);
    }

    /// Apply a batch of [`KnowledgeUpdate`]s produced by convention interpretation.
    pub fn apply_updates(&mut self, updates: &[KnowledgeUpdate], variant: &Variant) {
        for update in updates {
            match update {
                KnowledgeUpdate::NarrowPossibilities {
                    card_deck_index,
                    mask,
                } => {
                    self.narrow_inferred(*card_deck_index, *mask, variant);
                }
                KnowledgeUpdate::AddSignal {
                    card_deck_index,
                    signal,
                } => {
                    self.add_signal(*card_deck_index, signal.clone());
                }
            }
        }
    }

    /// Get the possible identities for a card (convention-inferred only).
    /// Returns None if no convention narrowing has been applied yet.
    pub fn possible_identities(&self, card_deck_index: CardDeckIndex) -> Option<Empathy> {
        self.inferred_identities[card_deck_index as usize]
    }

    /// Get the combined knowledge for a card: game-rule empathy merged with inferred identities.
    ///
    /// Game-rule empathy comes from the Deck (positive/negative clues, discards, stacks).
    /// Inferred identities narrow this based on convention interpretation.
    ///
    /// For cards in the player's own hand that haven't been identified by convention
    /// (i.e. not in `visible_cards`), only convention-inferred knowledge is returned.
    /// The player cannot see their own cards, so the omniscient deck empathy must not leak
    /// into their decision-making during search.
    pub fn combined_possible_identities(
        &self,
        card_deck_index: CardDeckIndex,
        table_state: &TableState,
        variant: &Variant,
    ) -> Empathy {
        let is_own_unseen = (self.own_hand >> card_deck_index) & 1 != 0
            && (self.visible_cards >> card_deck_index) & 1 == 0;

        if is_own_unseen {
            // Only convention-inferred knowledge; fully unknown if no clue has touched it.
            return self.inferred_identities[card_deck_index as usize]
                .unwrap_or_else(|| Empathy::all(variant));
        }

        let game_empathy = table_state.deck.get_global_empathy(card_deck_index);
        if let Some(inferred) = self.inferred_identities[card_deck_index as usize] {
            if let Some(combined) = game_empathy.narrow(inferred.as_bits()) {
                return combined;
            }
        }
        game_empathy
    }
}

#[cfg(test)]
pub fn knowledge_with_visible(player_index: usize, visible: &[(u8, u64)]) -> PlayerKnowledgeState {
    let mut k = PlayerKnowledgeState::new(player_index);
    for &(idx, mask) in visible {
        k.inferred_identities[idx as usize] = Some(Empathy::from_bits(mask).expect("zero mask in knowledge_with_visible"));
        k.visible_cards |= 1 << idx;
    }
    k
}

pub fn knowledge_for_hand(cards: &[u8]) -> PlayerKnowledgeState {
    let mut k = PlayerKnowledgeState::new(0);
    for &idx in cards {
        k.own_hand |= 1 << idx;
    }
    k
}

#[cfg(test)]
pub fn knowledge_with_empathy(
    card_deck_index: CardDeckIndex,
    possible_identities: VariantCardsBitField,
) -> PlayerKnowledgeState {
    let mut k = PlayerKnowledgeState::new(0);
    k.inferred_identities[card_deck_index as usize] = Some(Empathy::from_bits(possible_identities)
        .expect("zero mask in knowledge_with_empathy"));
    k.own_hand = 1 << card_deck_index;
    k
}