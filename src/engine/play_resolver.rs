use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::engine::knowledge_aware_game_state::KnowledgeAwareGameState;
use crate::game::card::{CardDeckIndex, VariantCardId};

/// The outcome of resolving a played card's identity.
pub enum ResolvedPlay {
    /// The card's identity is known exactly.
    Known(VariantCardId),
    /// The card is known-playable but its identity is ambiguous among multiple candidates
    /// (search mode only). Score advances via a phantom play; no specific stack is committed.
    KnownPlayableAmbiguous,
    /// No identity information — hidden-info fallback.
    Unknown,
}

/// What to draw after a play or discard.
pub enum DrawnCard {
    /// The deck is empty; no draw occurs.
    Empty,
    /// Draw a specific card with known identity (spectator/replay mode).
    Known { card_id: VariantCardId },
    /// Draw an unknown card; the state's internal draw logic handles advancement.
    Unknown,
}

/// Abstracts the two points of variation between search and replay:
/// card-identity resolution and draw mode.
///
/// All orchestration (collect_hypotheses, apply_cohort, resolve_pending) lives in
/// `KnowledgeAwareGameState::apply`; implementors supply only identity and draw.
pub trait PlayResolver {
    /// Determine the identity (or playable-but-ambiguous status) of a played card.
    fn resolve_play(
        &self,
        player_index: usize,
        card_deck_index: CardDeckIndex,
        state: &KnowledgeAwareGameState,
    ) -> ResolvedPlay;

    /// Determine the identity of a discarded card, if known.
    fn resolve_discard(
        &self,
        player_index: usize,
        card_deck_index: CardDeckIndex,
        state: &KnowledgeAwareGameState,
    ) -> Option<VariantCardId>;

    /// What card to draw after a play or discard.
    ///
    /// `deck_size` is the deck's current size after the play/discard (0 → `Empty`).
    /// `next_deck_index` is the state's draw cursor, used by replay to index `actual_deck`.
    fn draw_next(&mut self, deck_size: u8, next_deck_index: u8) -> DrawnCard;
}

// ── Search-mode resolver ──────────────────────────────────────────────────────

/// Search-mode resolver: uses truth-POV combined with empathy to determine play identity.
///
/// Wraps the root searcher's POV. Zero-alloc on construction.
pub struct TruthPovResolver<'a> {
    truth: &'a dyn PlayerPOV,
}

impl<'a> TruthPovResolver<'a> {
    pub fn new(truth: &'a dyn PlayerPOV) -> Self {
        Self { truth }
    }
}

impl PlayResolver for TruthPovResolver<'_> {
    fn resolve_play(
        &self,
        player_index: usize,
        card_deck_index: CardDeckIndex,
        state: &KnowledgeAwareGameState,
    ) -> ResolvedPlay {
        let knowledge = state.team_knowledge.player(player_index);
        let has_play_signal = knowledge.has_play_signal(card_deck_index);
        let combined = knowledge.combined_possible_identities(
            card_deck_index,
            &state.table_state,
            &state.static_data().variant,
        );
        let empathy_id = combined.known_card_id().or_else(|| {
            if has_play_signal {
                let playable = state.table_state.playable_cards(state.static_data());
                combined.narrow(playable).and_then(|e| e.known_card_id())
            } else {
                None
            }
        });
        // Truth override: prefer the root searcher's direct observation over empathy.
        let id = self.truth.card_identity(card_deck_index).or(empathy_id);
        if let Some(card_id) = id {
            ResolvedPlay::Known(card_id)
        } else if state.is_known_playable_play(player_index, card_deck_index, has_play_signal) {
            ResolvedPlay::KnownPlayableAmbiguous
        } else {
            ResolvedPlay::Unknown
        }
    }

    fn resolve_discard(
        &self,
        player_index: usize,
        card_deck_index: CardDeckIndex,
        state: &KnowledgeAwareGameState,
    ) -> Option<VariantCardId> {
        let num_players = state.static_data().number_of_players as usize;
        self.truth.card_identity(card_deck_index).or_else(|| {
            (0..num_players)
                .filter(|&obs| obs != player_index)
                .map(|obs| {
                    state
                        .team_knowledge
                        .player(obs)
                        .combined_possible_identities(
                            card_deck_index,
                            &state.table_state,
                            &state.static_data().variant,
                        )
                })
                .find(|e| e.is_exactly_known())
                .and_then(|e| e.known_card_id())
        })
    }

    fn draw_next(&mut self, _deck_size: u8, _next_deck_index: u8) -> DrawnCard {
        // update_with_unkown_card_draw handles the empty-deck guard internally.
        DrawnCard::Unknown
    }
}

// ── Replay-mode resolver ──────────────────────────────────────────────────────

/// Replay-mode resolver: uses the ground-truth deck for identity resolution.
///
/// Wraps a reference to the actual card sequence. Zero-alloc on construction.
pub struct GroundTruthResolver<'a> {
    actual_deck: &'a [VariantCardId],
}

impl<'a> GroundTruthResolver<'a> {
    pub fn new(actual_deck: &'a [VariantCardId]) -> Self {
        Self { actual_deck }
    }
}

impl PlayResolver for GroundTruthResolver<'_> {
    fn resolve_play(
        &self,
        _player_index: usize,
        card_deck_index: CardDeckIndex,
        _state: &KnowledgeAwareGameState,
    ) -> ResolvedPlay {
        ResolvedPlay::Known(self.actual_deck[card_deck_index as usize])
    }

    fn resolve_discard(
        &self,
        _player_index: usize,
        card_deck_index: CardDeckIndex,
        _state: &KnowledgeAwareGameState,
    ) -> Option<VariantCardId> {
        Some(self.actual_deck[card_deck_index as usize])
    }

    fn draw_next(&mut self, deck_size: u8, next_deck_index: u8) -> DrawnCard {
        if deck_size == 0 {
            return DrawnCard::Empty;
        }
        DrawnCard::Known {
            card_id: self.actual_deck[next_deck_index as usize],
        }
    }
}
