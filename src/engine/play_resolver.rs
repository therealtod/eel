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
/// All orchestration (`collect_hypotheses`, `apply_cohort`, `resolve_pending`) lives in
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

    /// Whether applying an action through this resolver advances the *real, observed*
    /// game (replay/live) rather than a hypothetical search rollout.
    ///
    /// Real progression reconciles each player's convention inference against the
    /// now-public empathy after every action (see
    /// [`KnowledgeAwareGameState::reconcile_inferred_with_public_empathy`]), keeping the
    /// engine's stored knowledge consistent with the rules of the game. Search rollouts
    /// deliberately *do not* reconcile: simulated knowledge is governed by the search's
    /// own POV-restriction machinery, which is independently tuned, and an extra
    /// reconciliation pass would perturb its phantom-play / known-play accounting.
    fn advances_real_game(&self) -> bool {
        false
    }
}

// ── Search-mode resolver ──────────────────────────────────────────────────────

/// Search-mode resolver: uses truth-POV combined with empathy to determine play identity.
///
/// Wraps the root searcher's POV. Zero-alloc on construction.
///
/// `observer` is the *planning self* in a recursive subjective sub-search (the inner player
/// asking "what would happen if I took this line?"). It supplies *direct physical sight* of a
/// teammate's card so the planner foresees the true outcome of a line it can see: a blind play
/// of a mis-pinned card resolves as the real strike it causes, and a blind discard of a card
/// the planner can plainly see — e.g. a critical on chop — registers its true identity for
/// lost-ceiling accounting. Both [`Self::resolve_play`] and [`Self::resolve_discard`] consult
/// it, but **only** as literal sight gated on `visible_cards`, never the observer's own
/// empathy: this is bounded by the search's effective-visibility restriction, so the observer
/// can never see the root player's hidden hand and thus cannot fabricate outcomes from an
/// X-ray view the actor had no justified basis to earn.
pub struct TruthPovResolver<'a> {
    truth: &'a dyn PlayerPOV,
    observer: Option<&'a dyn PlayerPOV>,
}

impl<'a> TruthPovResolver<'a> {
    pub fn new(truth: &'a dyn PlayerPOV) -> Self {
        Self {
            truth,
            observer: None,
        }
    }

    /// Construct a resolver that additionally lets the planning `observer` reveal discarded
    /// card identities it can see (see the struct-level note). `truth` remains the acting
    /// player's POV; `observer` is the inner player simulating the line.
    pub fn with_observer(truth: &'a dyn PlayerPOV, observer: &'a dyn PlayerPOV) -> Self {
        Self {
            truth,
            observer: Some(observer),
        }
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
                combined.narrow(playable).and_then(super::super::game::card::card_identity_mask::CardIdentityMask::known_card_id)
            } else {
                None
            }
        });
        // Truth override: prefer direct physical observation over empathy, so a teammate
        // simulating this line foresees the real outcome of a play it can see — a clean play
        // resolves as the true success, a foreseeable misplay (e.g. a bad touch) as a real
        // strike — instead of the always-optimistic phantom.
        //
        // A play is always of the actor's *own* hand, which the actor cannot see, so
        // `self.truth.card_identity` (the actor's POV) yields only the actor's *empathy*. A
        // convention mis-pin (e.g. a chop wrongly told it is the currently-playable card) can
        // collapse that empathy to a *false* identity — so consulting the actor first would
        // resolve the play to the believed card and mask the strike the misplay actually
        // causes. The planning observer (a teammate who can physically see the card) is ground
        // truth, so its *direct sight* takes precedence. We gate on the observer's
        // `visible_cards` to use only genuine sight, never the observer's own empathy: when the
        // actor is the root player (whose hand the observer cannot see, per the search's
        // effective-visibility restriction) the observer has no sight, so we fall through to
        // the actor's empathy and the root player's hidden hand never leaks. At the root truth
        // simulation `observer` is `None` and `self.truth` is itself a full-sight POV, so this
        // is unchanged there.
        let observer_sight = self.observer.and_then(|obs| {
            if obs.visible_cards() & (1u64 << card_deck_index) != 0 {
                obs.card_identity(card_deck_index)
            } else {
                None
            }
        });
        let id = observer_sight
            .or_else(|| self.truth.card_identity(card_deck_index))
            .or(empathy_id);
        if let Some(card_id) = id {
            ResolvedPlay::Known(card_id)
        } else if let Some(card_id) = knowledge.committed_play_identity(card_deck_index) {
            // Convention-committed fallback: a `Signal::Play` on this card names the
            // identity the convention says it has. Use it instead of falling through
            // to a stack-agnostic phantom advance so that downstream teammates'
            // empathy can narrow against the committed stack progression.
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
        // Resolve in order of decreasing justification, mirroring `resolve_play`:
        //   1. the planning observer's *direct physical sight*, if any (see `with_observer`);
        //   2. the acting player's truth-POV;
        //   3. the actor's *own* narrowed empathy.
        //
        // A discard is always of the actor's *own* hand, which the actor cannot see, so
        // `self.truth.card_identity` (the actor's POV) yields only the actor's *empathy* — and a
        // convention mis-pin can collapse that empathy to a *false* identity. Consulting the
        // actor first would then record the *believed* card leaving the game, mis-stating the
        // lost-ceiling consequence the discard actually causes (e.g. a foreseeably-critical chop
        // whose empathy was mis-pinned to a safe duplicate). A teammate simulating this line who
        // can physically see the card is ground truth, so its direct sight takes precedence.
        //
        // We gate on the observer's `visible_cards` to use only genuine sight, never the
        // observer's own empathy, and we never consult *other players'* team_knowledge directly.
        // This sight is bounded by the search's effective-visibility restriction: when the actor
        // is the root player (whose hand the observer cannot see) the observer has no sight, so
        // we fall through to the actor's empathy — the pile records an empathically-wide hole and
        // the root player's hidden hand never leaks, so downstream critical-mask reasoning gets
        // no free setup it couldn't have foreseen. At the root truth simulation `observer` is
        // `None` and `self.truth` is itself a full-sight POV, so this is unchanged there.
        let combined = state
            .team_knowledge
            .player(player_index)
            .combined_possible_identities(
                card_deck_index,
                &state.table_state,
                &state.static_data().variant,
            );
        let observer_sight = self.observer.and_then(|obs| {
            if obs.visible_cards() & (1u64 << card_deck_index) != 0 {
                obs.card_identity(card_deck_index)
            } else {
                None
            }
        });
        observer_sight
            .or_else(|| self.truth.card_identity(card_deck_index))
            .or_else(|| combined.known_card_id())
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
    #[must_use]
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

    fn advances_real_game(&self) -> bool {
        true
    }
}
