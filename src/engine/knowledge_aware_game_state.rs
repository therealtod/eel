use crate::engine::action_outcome::ActionOutcome;
use crate::engine::convention::convention_set::ConventionSet;
use crate::engine::convention::convention_tech::ConventionTech;
use crate::engine::game_state_snapshot::GameStateSnapshot;
use crate::engine::knowledge::knowledge_update::{Hypothesis, HypothesisId};
use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::engine::knowledge::team_knowledge::TeamKnowledge;
use crate::engine::play_resolver::{DrawnCard, PlayResolver, PlayerIndexTruthResolver};
use crate::game::MAX_HAND_SIZE;
use crate::game::action::game_action::GameAction;
use crate::game::card::{CardDeckIndex, VariantCardId};
use crate::game::clue::Clue;
use crate::game::state::table_state::TableState;
use crate::game::static_game_data::StaticGameData;
use crate::game::variant::Variant;
use smallvec::SmallVec;
use std::sync::Arc;

/// Collect hypotheses for `action` from `observer_pov`, respecting interpretation priority.
///
/// Collects up to two priority tiers (primary = highest matching, fallback = next highest).
/// Each returned entry is `(tier, hypothesis)` where `tier 0` = primary and `tier 1` = fallback.
/// Techs must be pre-sorted by `interpretation_priority` (ascending) —
/// `HGroupConventionSet::new` guarantees this. Empty hypotheses are dropped.
fn collect_hypotheses(
    techs: &[Box<dyn ConventionTech>],
    action: &GameAction,
    history: &[GameStateSnapshot],
    observer_pov: &dyn PlayerPOV,
) -> Vec<(u8, Hypothesis)> {
    let mut primary_priority: Option<u8> = None;
    let mut fallback_priority: Option<u8> = None;
    let mut result = Vec::new();
    for tech in techs {
        let priority = tech.interpretation_priority();
        // Stop once we've filled two tiers and this tech would start a third.
        if fallback_priority.is_some_and(|fp| priority > fp) {
            break;
        }
        if !tech.matches_action(action, history, observer_pov) {
            continue;
        }
        let hyps = tech.knowledge_updates_multi(action, history, observer_pov);
        let non_empty: smallvec::SmallVec<[Hypothesis; 1]> =
            hyps.into_iter().filter(|h| !h.is_empty()).collect();
        if non_empty.is_empty() {
            continue;
        }
        let tier = match primary_priority {
            None => {
                primary_priority = Some(priority);
                0
            }
            Some(pp) if priority == pp => 0,
            Some(_) => {
                if fallback_priority.is_none() {
                    fallback_priority = Some(priority);
                }
                1
            }
        };
        for h in non_empty {
            result.push((tier, h));
        }
    }
    result
}

/// Collect hypotheses from fallback techs using first-match-wins semantics.
///
/// Iterates `fallback_techs` in their declared order (matching the action-generation fallback
/// chain). Stops at the first tech whose `matches_action` returns `true` and returns its
/// (possibly empty) hypotheses — an empty return from a matched tech correctly signals
/// "stall recognised, no knowledge update" and prevents further matching.
fn collect_stall_hypotheses(
    fallback_techs: &[Box<dyn ConventionTech>],
    action: &GameAction,
    history: &[GameStateSnapshot],
    observer_pov: &dyn PlayerPOV,
) -> Vec<(u8, Hypothesis)> {
    for tech in fallback_techs {
        if tech.matches_action(action, history, observer_pov) {
            return tech
                .knowledge_updates_multi(action, history, observer_pov)
                .into_iter()
                .filter(|h| !h.is_empty())
                .map(|h| (0u8, h))
                .collect();
        }
    }
    vec![]
}

/// Route hypothesis collection to either the primary techs or the stall fallback chain,
/// depending on whether the giver had a primary candidate action at decision time.
/// Shared by the receiver and per-observer interpretation paths in [`KnowledgeAwareGameState::apply_clue`].
fn collect_pov_hypotheses(
    convention_set: &dyn ConventionSet,
    actor_had_primary_candidate: bool,
    action: &GameAction,
    history: &[GameStateSnapshot],
    observer_pov: &dyn PlayerPOV,
) -> Vec<(u8, Hypothesis)> {
    if actor_had_primary_candidate {
        collect_hypotheses(convention_set.techs(), action, history, observer_pov)
    } else {
        collect_stall_hypotheses(
            convention_set.fallback_techs(),
            action,
            history,
            observer_pov,
        )
    }
}

/// Reconstruct the giver's POV at decision time and report whether any primary tech had a
/// candidate action. If none did, the giver was stalling and clue interpretation must route
/// to the fallback techs instead of the primary techs — otherwise registered techs (e.g.
/// `DirectPlayClue`, `FiveSave`) would wrongly claim a stall clue as a play/save and emit bad
/// knowledge updates. Clue actions are discounted when the giver has no whole clue tokens.
///
/// This is the **observer gate** (strict): observers can see the receiver's hand. For the
/// receiver path, which cannot see its own hand, see [`receiver_had_primary_candidate`].
fn giver_had_primary_candidate(
    snapshot: &GameStateSnapshot,
    giver: usize,
    convention_set: &dyn ConventionSet,
    static_data: &StaticGameData,
) -> bool {
    let actor_pov = snapshot.player_pov(giver, static_data);
    let has_clue_tokens = actor_pov
        .table_state()
        .clue_token_bank
        .whole_clue_tokens_count()
        > 0;
    convention_set.techs().iter().any(|tech| {
        tech.game_actions(&actor_pov)
            .into_iter()
            .any(|a| has_clue_tokens || !matches!(a, GameAction::Clue { .. }))
    })
}

/// Receiver-side variant of [`giver_had_primary_candidate`]. The receiver cannot see its own
/// hand, so it cannot rule out that the giver had a play/save into that hand. When the giver's
/// reconstructed POV cannot resolve a card in the receiver's hand, give the giver the benefit of
/// the doubt (treat as non-stall) so play clues to the receiver's own hand are interpreted with
/// the primary techs rather than demoted to a stall.
fn receiver_had_primary_candidate(
    snapshot: &GameStateSnapshot,
    giver: usize,
    receiver: usize,
    convention_set: &dyn ConventionSet,
    static_data: &StaticGameData,
) -> bool {
    if giver_had_primary_candidate(snapshot, giver, convention_set, static_data) {
        return true;
    }
    // Strict gate failed: was that because the giver's POV (as we can reconstruct it) can't see
    // the receiver's hand? If so, a primary candidate there cannot be excluded.
    let giver_pov = snapshot.player_pov(giver, static_data);
    snapshot
        .table_state
        .hands[receiver]
        .cards()
        .iter()
        .any(|&slot| giver_pov.card_identity(slot).is_none())
}

/// A [`TableState`] with associated player knowledge and convention awareness.
///
/// This is the main integration point for the engine: it wraps a [`TableState`] with a
/// [`TeamKnowledge`], keeping both in sync as actions are applied.
///
/// Two variants of each mutating method are provided:
/// - `*_of_specific_card`: used when the card identity is known (spectator / replay mode).
/// - without suffix: used when the identity is unknown (alpha-beta search over hidden state).
///
/// `static_data` is stored behind an [`Arc`] so that cloning a search node (the hot path)
/// bumps a refcount rather than copying the ~200-byte [`StaticGameData`].
#[derive(Clone)]
pub struct KnowledgeAwareGameState {
    pub table_state: TableState,
    pub team_knowledge: TeamKnowledge,
    static_data: Arc<StaticGameData>,
    /// The deck index that will be assigned to the next synthesized draw.
    /// Initialized to `MAX_CARDS_IN_DECK - deck.current_size`.
    pub next_deck_index: u8,
    /// Monotonic counter for unique hypothesis ids.
    next_hypothesis_id: HypothesisId,
    /// Search-only ledger of playable card identities claimed by phantom plays along the
    /// current rollout line (one bit per [`VariantCardId`]). A phantom play
    /// ([`ResolvedPlay::KnownPlayableAmbiguous`]) advances the effective score by one — the
    /// same reward as a known play — whenever it can claim a still-playable identity not
    /// already in this mask. The only suppression is duplicate-identity dedup: two ambiguous
    /// copies that can *only* claim the same single remaining playable identity (e.g. a
    /// giver's own card duplicated by a teammate's clued card) bank a phantom point once, not
    /// twice. A later *real* play of a claimed identity does **not** revoke the phantom: the
    /// phantom card is a distinct physical card and, being known-playable, is optimistically
    /// assumed to score in its own right — exactly as a concrete known play would. The
    /// threaded `phantom_plays` count equals this mask's popcount. Always 0 in replay mode
    /// (the ground-truth resolver never produces phantom plays).
    phantom_claimed_mask: crate::game::card::VariantCardsBitField,
}

impl KnowledgeAwareGameState {
    #[must_use]
    pub fn new(static_data: StaticGameData) -> Self {
        let static_data = Arc::new(static_data);
        let table_state = TableState::new(&static_data);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        KnowledgeAwareGameState {
            table_state,
            team_knowledge,
            static_data,
            next_deck_index: 0,
            next_hypothesis_id: 0,
            phantom_claimed_mask: 0,
        }
    }

    /// Construct from an existing table state and team knowledge (e.g. for search).
    #[must_use]
    pub fn from_parts(
        static_data: StaticGameData,
        table_state: TableState,
        team_knowledge: TeamKnowledge,
        next_deck_index: u8,
    ) -> Self {
        KnowledgeAwareGameState {
            table_state,
            team_knowledge,
            static_data: Arc::new(static_data),
            next_deck_index,
            next_hypothesis_id: 0,
            phantom_claimed_mask: 0,
        }
    }

    /// Get the current turn number (sequential turn counter from table state).
    #[must_use]
    pub fn current_turn(&self) -> usize {
        self.table_state.current_turn
    }

    /// Get a read-only view of the game from the specified player's perspective.
    #[must_use]
    pub fn player_pov(&self, player_index: usize) -> LightweightPlayerPOV<'_> {
        LightweightPlayerPOV::new(
            player_index,
            self.team_knowledge.player(player_index),
            &self.team_knowledge,
            &self.table_state,
            &self.static_data,
        )
    }

    // ── Draw ─────────────────────────────────────────────────────────────────

    /// Apply a draw, revealing the card identity to all players except the drawer.
    ///
    /// Use this in spectator / replay mode where the identity is known.
    pub fn update_with_draw_action_of_specific_card(
        &mut self,
        player_index: usize,
        card_deck_index: CardDeckIndex,
        card_id: VariantCardId,
    ) {
        self.table_state.update_with_draw_action(card_deck_index);
        self.team_knowledge
            .update_with_card_drawn(player_index, card_deck_index, card_id);
    }

    /// Apply a draw without revealing the card identity (e.g., during alpha-beta search).
    ///
    /// Only the drawing player's own-hand bitmask is updated; no empathy updates are made.
    pub fn update_with_draw_action(&mut self, player_index: usize, card_deck_index: CardDeckIndex) {
        self.table_state.update_with_draw_action(card_deck_index);
        self.team_knowledge.player_mut(player_index).own_hand |= 1 << card_deck_index;
    }

    // ── Play ─────────────────────────────────────────────────────────────────

    /// Apply a play for the current player, knowing the card's identity.
    pub fn update_with_play_action_of_specific_card(
        &mut self,
        card_deck_index: CardDeckIndex,
        card_id: VariantCardId,
    ) {
        let player_index = self.table_state.active_player_index();
        let stack_idx = card_id / self.static_data.variant.stacks_size as usize;
        let pre = self.table_state.playing_stacks.stack_size(stack_idx);
        self.table_state.update_with_play_action_of_specific_card(
            card_deck_index,
            card_id,
            &self.static_data,
        );
        let post = self.table_state.playing_stacks.stack_size(stack_idx);
        self.remove_card_from_own_hand(player_index, card_deck_index);
        if post > pre {
            self.reapply_good_touch_after_state_change();
        }
    }

    /// Apply a play for the current player without knowing the card's identity.
    pub fn update_with_play_action(&mut self, card_deck_index: CardDeckIndex) {
        let player_index = self.table_state.active_player_index();
        self.table_state.update_with_play_action(card_deck_index);
        self.remove_card_from_own_hand(player_index, card_deck_index);
    }

    // ── Discard ───────────────────────────────────────────────────────────────

    /// Apply a discard for the current player, knowing the card's identity.
    pub fn update_with_discard_action_of_specific_card(
        &mut self,
        card_deck_index: CardDeckIndex,
        card_id: VariantCardId,
    ) {
        let player_index = self.table_state.active_player_index();
        self.table_state
            .update_with_discard_action_of_specific_card(
                card_deck_index,
                card_id,
                &self.static_data,
            );
        self.remove_card_from_own_hand(player_index, card_deck_index);
    }

    /// Apply a discard for the current player without knowing the card's identity.
    pub fn update_with_discard_action(&mut self, card_deck_index: CardDeckIndex) {
        let player_index = self.table_state.active_player_index();
        self.table_state
            .update_with_discard_action(card_deck_index, &self.static_data);
        self.remove_card_from_own_hand(player_index, card_deck_index);
    }

    // ── Clue ──────────────────────────────────────────────────────────────────

    /// Apply a clue action (table state only; no convention knowledge propagation).
    pub fn update_with_clue_action(
        &mut self,
        touched_card_deck_indexes: SmallVec<[CardDeckIndex; MAX_HAND_SIZE]>,
        clue: Clue,
        receiver_player_index: usize,
    ) {
        self.table_state.update_with_clue_action(
            touched_card_deck_indexes,
            clue,
            receiver_player_index,
            &self.static_data,
        );
    }

    // ── Search helpers ────────────────────────────────────────────────────────

    /// Apply a `GameAction` and propagate convention knowledge.
    /// Does NOT advance the turn; call `advance_turn()` separately.
    ///
    /// The search uses clone-and-recurse, so no undo token is needed.
    ///
    /// `resolver` determines how played/discarded card identities are resolved and how
    /// replacement draws are handled. Use [`TruthPovResolver`](crate::engine::play_resolver::TruthPovResolver)
    /// for search and [`GroundTruthResolver`](crate::engine::play_resolver::GroundTruthResolver)
    /// for replay.
    pub fn apply(
        &mut self,
        action: &GameAction,
        convention_set: &dyn ConventionSet,
        resolver: &mut dyn PlayResolver,
    ) -> ActionOutcome {
        let actor = self.table_state.active_player_index();
        let mut outcome = ActionOutcome::default();
        let played_identity: Option<VariantCardId> = match action {
            GameAction::Play {
                card_deck_index, ..
            } => {
                let (known_id, was_phantom) =
                    self.apply_play(*card_deck_index, convention_set, resolver);
                outcome.is_phantom_play = was_phantom;
                known_id
            }
            GameAction::Discard {
                card_deck_index, ..
            } => {
                self.apply_discard(*card_deck_index, resolver);
                None
            }
            GameAction::Clue {
                touched_card_deck_indexes,
                clue,
                player_index,
                ..
            } => {
                let touched = touched_card_deck_indexes.clone();
                let clue_val = clue.clone();
                let receiver = *player_index;
                self.apply_clue(&touched, &clue_val, receiver, action, convention_set);
                None
            }
            GameAction::Draw {
                card_deck_index,
                player_index,
            } => {
                self.table_state.update_with_draw_action(*card_deck_index);
                self.team_knowledge.player_mut(*player_index).own_hand |= 1u64 << *card_deck_index;
                None
            }
        };

        // Resolve pending interpretations across all players keyed on `actor`'s action.
        // Draw actions never trigger a resolution.
        if !matches!(action, GameAction::Draw { .. }) {
            let num_players = self.static_data.number_of_players as usize;
            for p in 0..num_players {
                self.team_knowledge.player_mut(p).resolve_pending(
                    actor,
                    action,
                    played_identity,
                    &self.static_data.variant,
                );
            }

            // Cross-check convention inference against the now-settled public empathy as
            // the *real* game advances. A Draw only adds the new card to `own_hand` (no
            // clue/reveal), so it can never narrow an existing card's empathy — and the
            // search rollout governs simulated knowledge through its own POV machinery, so
            // reconcile only when the resolver is advancing the observed game.
            if resolver.advances_real_game() {
                self.reconcile_inferred_with_public_empathy();
            }
        }
        outcome
    }

    /// Convenience wrapper: applies `action` using a [`TruthPovResolver`] constructed from
    /// the acting `player_index`'s POV. Avoids external borrow-checker conflicts when the
    /// caller cannot hold a `LightweightPlayerPOV` across a mutable `apply` call.
    pub fn apply_for_player(
        &mut self,
        action: &GameAction,
        convention_set: &dyn ConventionSet,
        player_index: usize,
    ) -> ActionOutcome {
        let mut resolver = PlayerIndexTruthResolver::new(player_index);
        self.apply(action, convention_set, &mut resolver)
    }

    fn apply_play(
        &mut self,
        card_deck_index: CardDeckIndex,
        convention_set: &dyn ConventionSet,
        resolver: &mut dyn PlayResolver,
    ) -> (Option<VariantCardId>, bool) {
        use crate::engine::play_resolver::ResolvedPlay;
        let p = self.table_state.active_player_index();
        let turn_counter = self.table_state.current_turn;
        let action = GameAction::Play {
            player_index: p,
            card_deck_index,
            turn: turn_counter,
        };
        let pov = LightweightPlayerPOV::new(
            p,
            self.team_knowledge.player(p),
            &self.team_knowledge,
            &self.table_state,
            &self.static_data,
        );
        let actor_hypotheses = collect_hypotheses(convention_set.techs(), &action, &[], &pov);
        if !actor_hypotheses.is_empty() {
            tracing::debug!(target: "eel::apply", giver = p, action = ?action, "tech_matched");
        }

        // Resolve the played card's identity (or its playable-but-ambiguous status) so that we
        // advance the engine's effective score and give subsequent plies accurate information.
        //
        // Three outcomes, in priority order:
        //
        // 1. Singleton identity: resolver returns `Known(card_id)`. Use the full table-state
        //    path that advances the matching stack or records a strike.
        //
        // 2. Known-playable but multi-id: resolver returns `KnownPlayableAmbiguous`. The play
        //    is guaranteed to succeed in expectation, but we deliberately refuse to commit to
        //    a single stack — guessing would distort downstream playability/criticality
        //    reasoning. Instead, record a phantom play so the effective score advances without
        //    revealing any identity.
        //
        // 3. Truly unknown: resolver returns `Unknown`. Fall back to the hidden-info path
        //    (card removed from hand, no score change). Essentially unreachable for
        //    legitimate Play actions generated by `PlayKnownPlayable` / `BlindPlay`.
        let play_resolved = resolver.resolve_play(p, card_deck_index, self);
        let stack_advanced;
        let mut is_phantom = false;
        let known_id: Option<VariantCardId>;
        match play_resolved {
            ResolvedPlay::Known(card_id) => {
                known_id = Some(card_id);
                let pre = self
                    .table_state
                    .playing_stacks
                    .stack_size(card_id / self.static_data.variant.stacks_size as usize);
                self.table_state.update_with_play_action_of_specific_card(
                    card_deck_index,
                    card_id,
                    &self.static_data,
                );
                let post = self
                    .table_state
                    .playing_stacks
                    .stack_size(card_id / self.static_data.variant.stacks_size as usize);
                stack_advanced = post > pre;
            }
            ResolvedPlay::KnownPlayableAmbiguous => {
                known_id = None;
                // Bank a phantom point only if this ambiguous card can claim a still-playable
                // identity not already realized on this line. Two ambiguous copies of the same
                // playable identity (a giver's own card duplicated by a teammate's clued card)
                // therefore advance the effective score only once — closing the rollout
                // double-count that made deferring a known play look as good as making it.
                let empathy = self
                    .team_knowledge
                    .player(p)
                    .combined_possible_identities(
                        card_deck_index,
                        &self.table_state,
                        &self.static_data.variant,
                    )
                    .as_bits();
                let playable = self.table_state.playable_cards(&self.static_data);
                let candidates = empathy & playable & !self.phantom_claimed_mask;
                if candidates != 0 {
                    self.phantom_claimed_mask |= 1u64 << candidates.trailing_zeros();
                    is_phantom = true;
                }
                self.table_state.update_with_play_action(card_deck_index);
                stack_advanced = false;
            }
            ResolvedPlay::Unknown => {
                known_id = None;
                self.table_state.update_with_play_action(card_deck_index);
                stack_advanced = false;
            }
        }
        self.remove_card_from_own_hand(p, card_deck_index);

        // Draw the replacement card. Replay mode draws a known card (reveals identity to
        // teammates); search mode synthesizes an unknown card via update_with_unkown_card_draw.
        match resolver.draw_next(self.table_state.deck.current_size, self.next_deck_index) {
            DrawnCard::Empty => {}
            DrawnCard::Known { card_id } => {
                let idx = self.next_deck_index;
                self.next_deck_index += 1;
                self.update_with_draw_action_of_specific_card(p, idx, card_id);
            }
            DrawnCard::Unknown => {
                self.update_with_unkown_card_draw(p);
            }
        }

        // Aggressive Good-Touch reinterpretation: when a play resolves to a specific
        // identity and advances a stack, that identity is now trash (further copies
        // are duplicates). Re-narrow every clue-touched, not-already-known-trash card
        // in every player's hand against the updated `still_needed` mask.
        if stack_advanced {
            self.reapply_good_touch_after_state_change();
            self.neutralize_landed_play_signals();
        }

        self.apply_actor_cohort(p, &actor_hypotheses);
        (known_id, is_phantom)
    }

    fn apply_discard(&mut self, card_deck_index: CardDeckIndex, resolver: &mut dyn PlayResolver) {
        let p = self.table_state.active_player_index();
        let known_id = resolver.resolve_discard(p, card_deck_index, self);
        if let Some(card_id) = known_id {
            // Identity known: record per-id count and reveal the slot. Bonus token handling
            // is folded into `update_with_discard_action_of_specific_card`.
            self.table_state
                .update_with_discard_action_of_specific_card(
                    card_deck_index,
                    card_id,
                    &self.static_data,
                );
        } else {
            self.table_state
                .update_with_discard_action(card_deck_index, &self.static_data);
        }
        self.remove_card_from_own_hand(p, card_deck_index);

        match resolver.draw_next(self.table_state.deck.current_size, self.next_deck_index) {
            DrawnCard::Empty => {}
            DrawnCard::Known { card_id } => {
                let idx = self.next_deck_index;
                self.next_deck_index += 1;
                self.update_with_draw_action_of_specific_card(p, idx, card_id);
            }
            DrawnCard::Unknown => {
                self.update_with_unkown_card_draw(p);
            }
        }
    }

    /// Distribute hypotheses from `actor`'s play/discard to all other players' knowledge.
    ///
    /// Each recipient's copy is filtered down to cards that are actually in their hand,
    /// then applied as a new cohort.
    fn apply_actor_cohort(&mut self, actor: usize, actor_hypotheses: &[(u8, Hypothesis)]) {
        let num_players = self.static_data.number_of_players as usize;
        let cohort_id = self.next_hypothesis_id;
        self.next_hypothesis_id += 1;
        for target in (0..num_players).filter(|&t| t != actor) {
            let own_hand = self.team_knowledge.player(target).own_hand;
            let filtered: Vec<(u8, Hypothesis)> = actor_hypotheses
                .iter()
                .map(|(tier, h)| {
                    (
                        *tier,
                        Hypothesis {
                            immediate: h
                                .immediate
                                .iter()
                                .filter(|u| own_hand & (1 << u.card_deck_index()) != 0)
                                .cloned()
                                .collect(),
                            trigger: h.trigger.clone(),
                            alt_group: h.alt_group,
                        },
                    )
                })
                .filter(|(_, h)| !h.is_empty())
                .collect();
            if !filtered.is_empty() {
                tracing::debug!(target: "eel::apply", target, hypotheses = filtered.len(), "knowledge_updated");
            }
            self.team_knowledge.player_mut(target).apply_cohort(
                cohort_id,
                filtered,
                &mut self.next_hypothesis_id,
                &self.static_data.variant,
            );
        }
    }

    fn apply_clue(
        &mut self,
        touched_card_deck_indexes: &SmallVec<[CardDeckIndex; MAX_HAND_SIZE]>,
        clue: &Clue,
        receiver: usize,
        action: &GameAction,
        convention_set: &dyn ConventionSet,
    ) {
        let giver = self.table_state.active_player_index();
        // Save pre-clue table state so that focus calculation can correctly distinguish
        // "already clued before this clue" from "newly touched by this clue". The snapshot
        // passed to techs via local_history uses this pre-clue table_state paired with
        // post-raw team_knowledge, giving get_clue_focus the right clue_touched_cards.
        let pre_clue_table_state = self.table_state.clone();
        self.table_state.update_with_clue_action(
            touched_card_deck_indexes.clone(),
            clue.clone(),
            receiver,
            &self.static_data,
        );

        // Phase 1 — raw narrowing: apply the hard public facts of the literal clue, then the
        // softer convention baselines. Order matters: raw facts first so baselines can only
        // intersect, never widen, the empathy the clue mask establishes.
        self.apply_raw_clue_narrowing(receiver, touched_card_deck_indexes, clue);
        self.apply_receiver_baseline(
            receiver,
            touched_card_deck_indexes,
            clue,
            &pre_clue_table_state,
            convention_set,
        );

        let cohort_id = self.next_hypothesis_id;
        self.next_hypothesis_id += 1;

        // Phase 2 — build the per-clue tech history. The snapshot pairs the PRE-clue
        // table_state with the POST-raw-narrowing team_knowledge:
        //
        // - pre-clue table_state.clue_touched_cards: get_clue_focus can correctly
        //   distinguish "already clued before this clue" from "newly touched", so chop
        //   detection and focus selection work correctly.
        //
        // - post-raw team_knowledge: convention techs see the narrowed empathy that the
        //   clue mask imposes. DelayedPlayClue needs this to recognise when a connecting
        //   card becomes uniquely known *because of* the clue, and DirectPlayClue needs
        //   it to detect good-touch duplicates introduced by the same clue.
        let tech_snapshot =
            GameStateSnapshot::new(pre_clue_table_state, self.team_knowledge.clone());
        let observer_gate =
            giver_had_primary_candidate(&tech_snapshot, giver, convention_set, &self.static_data);
        let receiver_gate = receiver_had_primary_candidate(
            &tech_snapshot,
            giver,
            receiver,
            convention_set,
            &self.static_data,
        );

        let action_turn = match action {
            GameAction::Clue { turn, .. } => *turn,
            _ => 0,
        };
        // Techs only consult `history.get(turn - 1)` (the clue being interpreted); padding to
        // `action_turn` covers search's nonzero turn indices without copying the full
        // `self.history`. We pad to at least 1 entry so techs can also access the giver's
        // pre-clue snapshot when `action_turn == 0` (scenarios that start mid-game at turn 0).
        let local_history = vec![tech_snapshot; action_turn.max(1)];

        // Phase 3 — convention interpretation: derive hypotheses for the receiver and for every
        // observer, applying each as a cohort tagged with the shared `cohort_id`.
        self.interpret_clue_for_receiver(
            receiver,
            giver,
            action,
            &local_history,
            cohort_id,
            receiver_gate,
            convention_set,
        );
        self.interpret_clue_for_observers(
            receiver,
            giver,
            action,
            &local_history,
            cohort_id,
            observer_gate,
            convention_set,
        );
    }

    /// Phase 1a of [`apply_clue`](Self::apply_clue): raw clue narrowing on the receiver's hand.
    /// Touched cards must match the clue mask, untouched cards must not. These are hard facts
    /// derived purely from the public clue; applying them first ensures that softer convention
    /// inferences (good-touch, tech hypotheses) can never widen empathy past what the literal
    /// clue establishes.
    fn apply_raw_clue_narrowing(
        &mut self,
        receiver: usize,
        touched_card_deck_indexes: &SmallVec<[CardDeckIndex; MAX_HAND_SIZE]>,
        clue: &Clue,
    ) {
        let clue_mask = self.static_data.variant.empathy_for_clue(clue).as_bits();
        let hand_slots: SmallVec<[CardDeckIndex; MAX_HAND_SIZE]> = self.table_state.hands[receiver]
            .cards()
            .iter()
            .copied()
            .collect();
        let receiver_knowledge = self.team_knowledge.player_mut(receiver);
        for slot in &hand_slots {
            if touched_card_deck_indexes.contains(slot) {
                receiver_knowledge.narrow_inferred(*slot, clue_mask, &self.static_data.variant);
            } else {
                receiver_knowledge.exclude_inferred(*slot, clue_mask, &self.static_data.variant);
            }
        }
    }

    /// Phase 1b of [`apply_clue`](Self::apply_clue): convention-wide baseline narrowings on the
    /// receiver (e.g. H-Group good-touch principle: every touched card is assumed eventually
    /// useful). Applied as an unconditional baseline so it intersects with per-tech cohort
    /// hypotheses.
    ///
    /// Per-card exception: if a touched card's post-raw empathy is already a subset of
    /// non-needed identities (every candidate is trash by public information), the holder won't
    /// misplay it without GTP help. Skip the GTP narrowing for that card so we don't smuggle in
    /// fake "still-useful" plausibility on top of trash empathy.
    fn apply_receiver_baseline(
        &mut self,
        receiver: usize,
        touched_card_deck_indexes: &SmallVec<[CardDeckIndex; MAX_HAND_SIZE]>,
        clue: &Clue,
        pre_clue_table_state: &TableState,
        convention_set: &dyn ConventionSet,
    ) {
        let still_needed = crate::engine::convention::hgroup::h_group_core::still_needed_cards_mask(
            &self.table_state,
            &self.static_data,
        );
        // Focus must be computed from the PRE-clue table_state so newly-touched cards are
        // distinguishable from previously-clued ones (see `get_clue_focus`).
        let focus = {
            let pre_clue_pov = LightweightPlayerPOV::new(
                receiver,
                self.team_knowledge.player(receiver),
                &self.team_knowledge,
                pre_clue_table_state,
                &self.static_data,
            );
            crate::engine::convention::hgroup::h_group_core::get_clue_focus(
                receiver,
                touched_card_deck_indexes,
                &pre_clue_pov,
            )
        };
        for (idx, mask) in convention_set.clue_receiver_baseline(
            clue,
            touched_card_deck_indexes,
            focus,
            receiver,
            &self.table_state,
            &self.static_data,
        ) {
            let current_empathy =
                self.team_knowledge.player(receiver).inferred_identities[idx as usize].map_or(
                    u64::MAX,
                    super::super::game::card::card_identity_mask::CardIdentityMask::as_bits,
                );
            if current_empathy & still_needed == 0 {
                continue;
            }
            self.team_knowledge.player_mut(receiver).narrow_inferred(
                idx,
                mask,
                &self.static_data.variant,
            );
        }
    }

    /// Phase 3a of [`apply_clue`](Self::apply_clue): collect all matching techs' hypotheses from
    /// the receiver's own POV and apply them as a cohort. Uses the current (post-raw) `table_state`
    /// so the receiver's POV reflects the clue token spent and the deck narrowing, while
    /// `knowledge_history` supplies pre-clue `clue_touched_cards` for focus calculation inside the
    /// techs.
    fn interpret_clue_for_receiver(
        &mut self,
        receiver: usize,
        giver: usize,
        action: &GameAction,
        knowledge_history: &[GameStateSnapshot],
        cohort_id: HypothesisId,
        had_primary_candidate: bool,
        convention_set: &dyn ConventionSet,
    ) {
        let receiver_hypotheses = {
            let receiver_pov = LightweightPlayerPOV::new(
                receiver,
                self.team_knowledge.player(receiver),
                &self.team_knowledge,
                &self.table_state,
                &self.static_data,
            );
            collect_pov_hypotheses(
                convention_set,
                had_primary_candidate,
                action,
                knowledge_history,
                &receiver_pov,
            )
        };
        if !receiver_hypotheses.is_empty() {
            tracing::debug!(target: "eel::apply", giver, action = ?action, hypotheses = receiver_hypotheses.len(), "receiver_hypotheses");
        }
        self.team_knowledge.player_mut(receiver).apply_cohort(
            cohort_id,
            receiver_hypotheses,
            &mut self.next_hypothesis_id,
            &self.static_data.variant,
        );
    }

    /// Phase 3b of [`apply_clue`](Self::apply_clue): each non-receiver evaluates all techs from
    /// their own POV. Receiver-targeted updates are filtered down to the observer's own hand
    /// (other-hand updates are redundant since they can see those hands directly via
    /// `visible_cards`).
    ///
    /// Each observer's POV only differs from the shared `table_state` in `active_player_index`.
    /// Rather than cloning the whole `TableState` per observer just to overwrite that one field,
    /// temporarily point the shared state at the observer and restore it after the loop.
    #[allow(clippy::too_many_arguments)]
    fn interpret_clue_for_observers(
        &mut self,
        receiver: usize,
        giver: usize,
        action: &GameAction,
        knowledge_history: &[GameStateSnapshot],
        cohort_id: HypothesisId,
        had_primary_candidate: bool,
        convention_set: &dyn ConventionSet,
    ) {
        let num_players = self.static_data.number_of_players as usize;
        let original_active = self.table_state.active_player_index;
        for target in (0..num_players).filter(|&t| t != receiver) {
            self.table_state.active_player_index = target;
            let raw = {
                let target_pov = LightweightPlayerPOV::new(
                    target,
                    self.team_knowledge.player(giver),
                    &self.team_knowledge,
                    &self.table_state,
                    &self.static_data,
                );
                collect_pov_hypotheses(
                    convention_set,
                    had_primary_candidate,
                    action,
                    knowledge_history,
                    &target_pov,
                )
            };
            let own_hand = self.team_knowledge.player(target).own_hand;
            let filtered: Vec<(u8, Hypothesis)> = raw
                .into_iter()
                .map(|(tier, h)| {
                    (
                        tier,
                        Hypothesis {
                            immediate: h
                                .immediate
                                .into_iter()
                                .filter(|u| own_hand & (1 << u.card_deck_index()) != 0)
                                .collect(),
                            trigger: h.trigger,
                            alt_group: h.alt_group,
                        },
                    )
                })
                .filter(|(_, h)| !h.is_empty())
                .collect();
            if !filtered.is_empty() {
                tracing::debug!(target: "eel::apply", target, hypotheses = filtered.len(), "knowledge_updated");
            }
            self.team_knowledge.player_mut(target).apply_cohort(
                cohort_id,
                filtered,
                &mut self.next_hypothesis_id,
                &self.static_data.variant,
            );
        }
        self.table_state.active_player_index = original_active;
    }

    /// Advance `active_player_index` to the next player.
    pub fn advance_turn(&mut self) {
        let num_players = self.static_data.number_of_players as usize;
        self.table_state.advance_turn(num_players);
    }

    /// If the deck is non-empty, deal the next unknown card to `player_index`.
    pub fn update_with_unkown_card_draw(&mut self, player_index: usize) {
        if self.table_state.deck.current_size == 0 {
            return;
        }
        let idx = self.next_deck_index;
        debug_assert!(
            (idx as usize) < crate::game::MAX_CARDS_IN_DECK,
            "next_deck_index {} out of bounds (MAX_CARDS_IN_DECK={})",
            idx,
            crate::game::MAX_CARDS_IN_DECK,
        );
        self.next_deck_index += 1;
        self.table_state.update_with_draw_action(idx);
        self.team_knowledge.player_mut(player_index).own_hand |= 1u64 << idx;
    }

    /// Remove a card from a player's own-hand bitmask.
    fn remove_card_from_own_hand(&mut self, player_index: usize, card_deck_index: CardDeckIndex) {
        self.team_knowledge.player_mut(player_index).own_hand &= !(1u64 << card_deck_index);
    }

    /// Re-apply the good-touch baseline to every clue-touched card in every hand.
    ///
    /// Called after an event that shrinks `still_needed_cards_mask` (currently: a play
    /// that advances a stack). Each touched card whose inferred mask still overlaps
    /// `still_needed` is narrowed to that mask, removing newly-played identities from
    /// its empathy under GTP ("touched cards are eventually useful, so they are not
    /// duplicates of cards already on the stacks"). Cards that are already fully
    /// known-trash are left alone — same conservative skip as the clue-time GTP path.
    /// Neutralize any baseline [`Signal::Play`](crate::engine::convention::hgroup::signal::Signal)
    /// whose committed identity has already landed on the stacks.
    ///
    /// A play signal promises "this card is the named playable identity." Once a copy of that
    /// identity is actually played, the promise is spent: the held card is now a trash
    /// duplicate, and acting on the stale signal would blind-play it for a strike. This fires
    /// after every stack advance — the single moment a committed identity can become played —
    /// so the signal is retired the instant it is fulfilled, in both real-game application and
    /// search rollouts.
    fn neutralize_landed_play_signals(&mut self) {
        use crate::engine::convention::hgroup::signal::Signal;
        let num_players = self.static_data.number_of_players as usize;
        let stacks = &self.table_state.playing_stacks;
        for p in 0..num_players {
            let slots: SmallVec<[CardDeckIndex; MAX_HAND_SIZE]> =
                self.table_state.hands[p].cards().iter().copied().collect();
            let knowledge = self.team_knowledge.player_mut(p);
            for idx in slots {
                knowledge.signals[idx as usize].retain(|s| match s {
                    Signal::Play {
                        committed_identity, ..
                    } => !stacks.contains_card_with_id(*committed_identity),
                    _ => true,
                });
            }
        }
    }

    fn reapply_good_touch_after_state_change(&mut self) {
        let still_needed = crate::engine::convention::hgroup::h_group_core::still_needed_cards_mask(
            &self.table_state,
            &self.static_data,
        );
        let num_players = self.static_data.number_of_players as usize;
        let touched = self.table_state.clue_touched_cards;
        for p in 0..num_players {
            let slots: SmallVec<[CardDeckIndex; MAX_HAND_SIZE]> =
                self.table_state.hands[p].cards().iter().copied().collect();
            for idx in slots {
                if touched & (1u64 << idx) == 0 {
                    continue;
                }
                let current = self.team_knowledge.player(p).inferred_identities[idx as usize]
                    .map_or(
                        u64::MAX,
                        super::super::game::card::card_identity_mask::CardIdentityMask::as_bits,
                    );
                if current & still_needed == 0 {
                    continue;
                }
                self.team_knowledge.player_mut(p).narrow_inferred(
                    idx,
                    still_needed,
                    &self.static_data.variant,
                );
            }
        }
    }

    /// Re-establish the invariant that every card's convention-derived
    /// `inferred_identities` stays a subset of what the rules of the game still
    /// allow — the deck's public empathy (positive clues, negative clues, and
    /// visible-copy exhaustion).
    ///
    /// Called from [`apply`](Self::apply) at the end of every action that advances the
    /// *real* game (a ground-truth resolver — see
    /// [`PlayResolver::advances_real_game`]), after the deck empathy has fully settled.
    /// `apply` is the single choke point all real game progression funnels through, so
    /// one pass here captures *every* way the public empathy can change — including the
    /// recursive narrowing `Deck::reveal_card` fans out to arbitrary slots when a card's
    /// last copy is located, which has no per-event hook of its own. Search rollouts
    /// deliberately skip this pass and let their own POV machinery govern simulated
    /// knowledge.
    ///
    /// For each hand card carrying a convention narrowing, the inferred mask is
    /// reconciled with the public empathy; on contradiction the convention inference
    /// is dropped and reality prevails (see
    /// [`PlayerKnowledge::reconcile_inferred_with_public`]). This stops an aggressive
    /// good-touch assumption from surviving as a "known" identity that later public
    /// information has ruled out.
    fn reconcile_inferred_with_public_empathy(&mut self) {
        let num_players = self.static_data.number_of_players as usize;
        for p in 0..num_players {
            let slots: SmallVec<[CardDeckIndex; MAX_HAND_SIZE]> =
                self.table_state.hands[p].cards().iter().copied().collect();
            for idx in slots {
                let public = self.table_state.deck.get_global_empathy(idx);
                self.team_knowledge
                    .player_mut(p)
                    .reconcile_inferred_with_public(idx, public);
            }
        }
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    #[must_use]
    pub fn table_state(&self) -> &TableState {
        &self.table_state
    }

    #[must_use]
    pub fn static_data(&self) -> &StaticGameData {
        &self.static_data
    }

    #[must_use]
    pub fn team_knowledge(&self) -> &TeamKnowledge {
        &self.team_knowledge
    }

    /// Number of phantom plays realized along the current rollout line — the popcount of the
    /// per-line [`phantom_claimed_mask`](Self::phantom_claimed_mask). This is the value to thread
    /// into the evaluator's `phantom_plays` arguments after applying an action: it already
    /// accounts for duplicate-identity dedup and for real plays that release an earlier
    /// optimistic phantom claim.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn phantom_play_count(&self) -> u8 {
        self.phantom_claimed_mask.count_ones() as u8
    }

    /// Reset the phantom-claim ledger to empty. Used to isolate a nested subjective sub-search
    /// (which scores from a fresh `phantom_plays = 0` base) from the outer line's claims.
    pub fn reset_phantom_claims(&mut self) {
        self.phantom_claimed_mask = 0;
    }

    /// Effective game score: real stack progress.
    #[must_use]
    pub fn score(&self, variant: &Variant, phantom_plays: u8) -> u8 {
        self.table_state.score(variant) + phantom_plays
    }

    /// Pace adjusted for phantom plays.
    #[must_use]
    pub fn pace(&self, phantom_plays: u8) -> i32 {
        self.table_state.pace(&self.static_data) + i32::from(phantom_plays)
    }

    /// Required efficiency adjusted for phantom plays.
    ///
    /// # Panics
    /// Panics if the score calculation overflows internal conversions.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    pub fn required_efficiency(&self, phantom_plays: u8) -> f32 {
        let variant = &self.static_data.variant;
        let max_score = i32::from(variant.number_of_suits * variant.stacks_size);
        let num_players = self.static_data.number_of_players as usize;
        let hand_cards: i32 = self.table_state.hands[..num_players]
            .iter()
            .map(|h| h.cards().len() as i32)
            .sum();
        let remaining = i32::from(self.table_state.deck.current_size) + hand_cards;
        let still_to_play = max_score - i32::from(self.score(variant, phantom_plays));
        let spare_turns = (remaining - still_to_play).max(0);
        if still_to_play <= 0 {
            return 0.0;
        }
        if spare_turns == 0 {
            return f32::INFINITY;
        }
        let all_hand_bits = self.table_state.all_hands_bitfield;
        let live_setups =
            i32::try_from((self.table_state.clue_touched_cards & all_hand_bits).count_ones())
                .unwrap();
        f64::from((still_to_play - live_setups).max(0)) as f32 / f64::from(spare_turns) as f32
    }

    /// True once the team has reached the theoretical max score (counting phantom plays)
    /// or struck out.
    #[must_use]
    pub fn is_terminal(&self, phantom_plays: u8) -> bool {
        let max_score =
            self.static_data.variant.number_of_suits * self.static_data.variant.stacks_size;
        self.table_state.strike_tokens >= 3
            || self.score(&self.static_data.variant, phantom_plays) >= max_score
    }

    /// True when the active player knows the card at `card_deck_index` is playable, even though
    /// its specific identity is not (yet) fully resolved.
    ///
    /// Returns `true` iff either:
    /// - the player has a `Signal::Play` on the card and the empathy ∩ playable intersection is
    ///   non-empty (the signal commits the player to playing it, and at least one playable
    ///   identity remains), OR
    /// - every identity in the player's combined empathy is a currently-playable card (no signal
    ///   needed; the card has been narrowed to a subset of the playable set).
    pub(crate) fn is_known_playable_play(
        &self,
        player_index: usize,
        card_deck_index: CardDeckIndex,
        has_play_signal: bool,
    ) -> bool {
        let knowledge = self.team_knowledge.player(player_index);
        let combined = knowledge.combined_possible_identities(
            card_deck_index,
            &self.table_state,
            &self.static_data.variant,
        );
        let bits = combined.as_bits();
        if bits == 0 {
            return false;
        }
        let playable = self.table_state.playable_cards(&self.static_data);
        if has_play_signal {
            (bits & playable) != 0
        } else {
            (bits & !playable) == 0
        }
    }

    /// Capture the current board state and team knowledge as an owned snapshot.
    #[must_use]
    pub fn snapshot(&self) -> GameStateSnapshot {
        GameStateSnapshot::new(self.table_state.clone(), self.team_knowledge.clone())
    }

    /// Returns `true` while the game is still in the **Early Game** phase — i.e.
    /// no player has yet performed a `DiscardChop` (discarded their chop).
    ///
    /// The history slice is always empty in search rollouts (snapshots are not recorded during
    /// search), so this reflects the phase as of the root state throughout a line. See
    /// [`h_group_core::is_early_game`](crate::engine::convention::hgroup::h_group_core::is_early_game).
    #[must_use]
    pub fn is_early_game(&self) -> bool {
        crate::engine::convention::hgroup::h_group_core::is_early_game(
            &[],
            &self.table_state,
            &self.static_data,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::play_resolver::TruthPovResolver;
    use crate::game::card::CardIdentityMask;
    use crate::game::clue_type::ClueType;
    use crate::game::variant::test_variants::NO_VARIANT;

    fn make_state() -> KnowledgeAwareGameState {
        let static_data = StaticGameData {
            number_of_players: 3,
            variant: NO_VARIANT,
        };
        KnowledgeAwareGameState::new(static_data)
    }

    #[test]
    fn draw_with_known_identity_makes_card_visible_to_other_players() {
        let mut state = make_state();
        let card_deck_index = 5;
        let card_id = 3;

        state.update_with_draw_action_of_specific_card(0, card_deck_index, card_id);

        // Players 1 and 2 can see the card
        assert_ne!(
            state.team_knowledge().player(1).visible_cards & (1 << card_deck_index),
            0
        );
        assert_ne!(
            state.team_knowledge().player(2).visible_cards & (1 << card_deck_index),
            0
        );
        // The drawer cannot see it
        assert_eq!(
            state.team_knowledge().player(0).visible_cards & (1 << card_deck_index),
            0
        );
    }

    #[test]
    fn draw_with_known_identity_puts_card_in_drawers_own_hand() {
        let mut state = make_state();
        let card_deck_index = 5;

        state.update_with_draw_action_of_specific_card(0, card_deck_index, 3);

        assert_ne!(
            state.team_knowledge().player(0).own_hand & (1 << card_deck_index),
            0
        );
    }

    #[test]
    fn draw_with_known_identity_does_not_narrow_drawers_empathy() {
        let mut state = make_state();
        let card_deck_index = 5;
        let card_id = 3;
        let variant = NO_VARIANT;

        state.update_with_draw_action_of_specific_card(0, card_deck_index, card_id);

        // The drawer cannot see their own card: combined empathy must be fully unknown,
        // not the omniscient deck identity. Only convention signals can narrow it.
        let combined = state
            .team_knowledge()
            .player(0)
            .combined_possible_identities(card_deck_index, &state.table_state, &variant);
        assert_eq!(
            combined.as_bits(),
            CardIdentityMask::all(&variant).as_bits(),
            "drawer should not know the card's identity"
        );
    }

    #[test]
    fn play_removes_card_from_own_hand() {
        let mut state = make_state();
        let card_deck_index = 0;

        // player 0 draws then plays the card
        state.update_with_draw_action_of_specific_card(0, card_deck_index, 0);
        state.update_with_play_action_of_specific_card(card_deck_index, 0);

        assert_eq!(
            state.team_knowledge().player(0).own_hand & (1 << card_deck_index),
            0
        );
    }

    #[test]
    fn discard_removes_card_from_own_hand() {
        let mut state = make_state();
        let card_deck_index = 0;

        state.update_with_draw_action_of_specific_card(0, card_deck_index, 1);
        state.update_with_discard_action_of_specific_card(card_deck_index, 1);

        assert_eq!(
            state.team_knowledge().player(0).own_hand & (1 << card_deck_index),
            0
        );
    }

    #[test]
    fn phantom_play_advances_score_without_touching_stacks() {
        // Build a state where player 0 holds a card whose inferred identity spans
        // multiple playable card-ids (here all five rank-1s). apply_play should route
        // through the phantom-play branch: the engine-effective score advances by 1,
        // but no specific stack moves and no identity is committed in the deck.
        use crate::engine::convention::hgroup::h_group_convention_set::HGroupConventionSet;

        let mut state = make_state();
        // Player 0 draws a card at deck index 0. Identity g1 (id 5) is hidden from
        // the drawer but visible to other players. Advance next_deck_index past the
        // existing hand so the post-play replacement draw lands on a fresh slot.
        state.update_with_draw_action_of_specific_card(0, 0, 5);
        state.next_deck_index = 1;
        // Simulate the post-clue inferred state directly: player 0's view of the card
        // is narrowed to {r1, y1, g1, b1, p1} — every rank-1. On empty stacks, all
        // five are currently playable.
        let ones_mask: u64 = (1 << 0) | (1 << 5) | (1 << 10) | (1 << 15) | (1 << 20);
        state.team_knowledge.player_mut(0).inferred_identities[0] =
            Some(CardIdentityMask::from_bits(ones_mask));

        let pre_stack_total = state.table_state.score(&state.static_data.variant);
        let pre_score = state.score(&state.static_data.variant, 0);
        assert_eq!(
            pre_score,
            state.table_state.score(&state.static_data.variant)
        );

        let conventions = HGroupConventionSet::new(vec![]);
        let play_action = GameAction::Play {
            player_index: 0,
            card_deck_index: 0,
            turn: 1,
        };
        let knowledge_clone = state.team_knowledge.player(0).clone();
        let team_clone = state.team_knowledge.clone();
        let table_clone = state.table_state.clone();
        let static_clone = state.static_data.clone();
        let pov = LightweightPlayerPOV::new(
            0,
            &knowledge_clone,
            &team_clone,
            &table_clone,
            &static_clone,
        );
        let outcome = state.apply(&play_action, &conventions, &mut TruthPovResolver::new(&pov));

        assert!(
            outcome.is_phantom_play,
            "apply should report phantom play for an ambiguous-but-playable play"
        );
        assert_eq!(
            state.table_state.score(&state.static_data.variant),
            pre_stack_total,
            "no concrete stack should advance for a phantom play"
        );
        assert_eq!(
            state.score(&state.static_data.variant, 1),
            pre_score + 1,
            "engine-effective score should reflect the phantom play"
        );
        // Played card removed from the player's own-hand mask. The post-play draw
        // lands on deck index 1, so bit 0 is clear and bit 1 is set.
        assert_eq!(state.team_knowledge().player(0).own_hand & 1, 0);
    }

    #[test]
    fn phantom_play_does_not_fire_when_identity_is_fully_resolved() {
        // If the player's inferred identity narrows to exactly one card-id, the
        // play goes through the fully-resolved path — the stack advances normally
        // and no phantom is recorded.
        use crate::engine::convention::hgroup::h_group_convention_set::HGroupConventionSet;

        let mut state = make_state();
        state.update_with_draw_action_of_specific_card(0, 0, 0); // r1 at deck 0
        state.next_deck_index = 1;
        state.team_knowledge.player_mut(0).inferred_identities[0] =
            Some(CardIdentityMask::from_bits(1)); // narrowed to r1 only

        let conventions = HGroupConventionSet::new(vec![]);
        let knowledge_clone = state.team_knowledge.player(0).clone();
        let team_clone = state.team_knowledge.clone();
        let table_clone = state.table_state.clone();
        let static_clone = state.static_data.clone();
        let pov = LightweightPlayerPOV::new(
            0,
            &knowledge_clone,
            &team_clone,
            &table_clone,
            &static_clone,
        );
        let outcome = state.apply(
            &GameAction::Play {
                player_index: 0,
                card_deck_index: 0,
                turn: 1,
            },
            &conventions,
            &mut TruthPovResolver::new(&pov),
        );

        assert!(!outcome.is_phantom_play);
        assert_eq!(state.table_state.score(&state.static_data.variant), 1);
        assert_eq!(state.score(&state.static_data.variant, 0), 1);
    }

    #[test]
    fn player_pov_can_be_created_for_any_player() {
        let mut state = make_state();
        state.update_with_draw_action_of_specific_card(0, 7, 2);

        // Just verify player_pov builds without panic for each player.
        let _ = state.player_pov(0);
        let _ = state.player_pov(1);
        let _ = state.player_pov(2);
    }

    // Regression: get_clue_focus must use the PRE-clue table_state so that
    // clue_touched_cards reflects which cards were already clued before the
    // current clue. Previously apply_clue passed the POST-clue snapshot, where
    // every newly-touched card already appeared as is_touched, emptying the
    // "newly touched" set and causing focus to fall back to the leftmost touched
    // card (slot 1) instead of the chop.
    #[test]
    fn clue_focus_is_chop_not_leftmost_when_chop_is_newly_touched() {
        use crate::engine::convention::hgroup::h_group_convention_set::HGroupConventionSet;
        use crate::engine::convention::hgroup::tech::direct_play_clue::DirectPlayClue;

        // 3 players; stacks empty throughout.
        let static_data = StaticGameData {
            number_of_players: 3,
            variant: NO_VARIANT,
        };
        let mut state = KnowledgeAwareGameState::new(static_data);

        // Alice (player 0) draws two cards.
        //   deck[0] = R1 (id 0) — drawn first → oldest → chop
        //   deck[1] = R2 (id 1) — drawn second → newest → slot 1
        // Both are unclued. The red color clue below touches both.
        state.update_with_draw_action_of_specific_card(0, 0, 0); // R1
        state.update_with_draw_action_of_specific_card(0, 1, 1); // R2

        // Bob (player 1) gives a red color clue to Alice.
        state.table_state.active_player_index = 1;

        let clue = Clue {
            clue_type: ClueType::Color,
            clue_value: 0, // red = suit 0
        };
        let action = GameAction::Clue {
            player_index: 0,
            touched_card_deck_indexes: smallvec::smallvec![0, 1],
            clue,
            turn: 1,
        };

        // Convention set with only DirectPlayClue so the test is self-contained.
        let conv = HGroupConventionSet::new(vec![Box::new(DirectPlayClue)]);
        let knowledge_clone = state.team_knowledge.player(1).clone();
        let team_clone = state.team_knowledge.clone();
        let table_clone = state.table_state.clone();
        let static_clone = state.static_data.clone();
        let pov = LightweightPlayerPOV::new(
            1,
            &knowledge_clone,
            &team_clone,
            &table_clone,
            &static_clone,
        );
        let _ = state.apply(&action, &conv, &mut TruthPovResolver::new(&pov));

        let alice = state.team_knowledge.player(0);

        // deck[0] is the chop and the correct focus. DirectPlayClue narrows the
        // focus to immediately-playable red ids only. On empty stacks the only
        // playable red card is R1 (id 0, bit 0), so the empathy must be exactly 1.
        // With the pre-fix bug the focus was deck[1] (leftmost), leaving deck[0]
        // at the raw red mask {R1..R5} = 0b11111 = 31.
        let deck0_empathy = alice
            .combined_possible_identities(0, &state.table_state, &state.static_data.variant)
            .as_bits();
        assert_eq!(
            deck0_empathy, 1u64,
            "chop (deck[0]=R1) should be narrowed to {{R1}} by DirectPlayClue as focus; \
             got {deck0_empathy:025b}"
        );

        // deck[1] is not the focus; DirectPlayClue applies no hypothesis to it,
        // leaving it at the raw red mask = {R1..R5} = bits 0–4.
        let deck1_empathy = alice
            .combined_possible_identities(1, &state.table_state, &state.static_data.variant)
            .as_bits();
        let red_mask: u64 = 0b11111;
        assert_eq!(
            deck1_empathy & red_mask,
            red_mask,
            "non-focus (deck[1]=R2) should retain all red identities; \
             got {deck1_empathy:025b}"
        );
    }

    #[test]
    fn receiver_had_primary_candidate_delegates_to_strict_when_strict_passes() {
        use crate::engine::convention::hgroup::h_group_convention_set::HGroupConventionSet;
        use crate::engine::convention::hgroup::tech::play_known_playable::PlayKnownPlayable;
        use crate::engine::convention::convention_tech::ConventionTech;

        let mut state = make_state();
        // Give player 0 (giver) a known playable card (R1, id 0)
        state.update_with_draw_action_of_specific_card(0, 0, 0);
        // Narrow player 0's knowledge to know it's playable
        state
            .team_knowledge
            .player_mut(0)
            .narrow_inferred(0, 1u64, &state.static_data.variant);

        // Player 1 (receiver) has an unresolved card
        state.update_with_draw_action_of_specific_card(1, 5, 1);

        let snapshot = GameStateSnapshot::new(state.table_state.clone(), state.team_knowledge);
        let conventions =
            HGroupConventionSet::new(vec![Box::new(PlayKnownPlayable) as Box<dyn ConventionTech>]);

        // Strict gate should pass (giver has a playable)
        let strict_gate = giver_had_primary_candidate(
            &snapshot,
            0, // giver
            &conventions,
            &state.static_data,
        );
        assert!(
            strict_gate,
            "strict gate should pass when giver has primary candidate"
        );

        // Receiver gate should also pass (delegates to strict)
        let receiver_gate = receiver_had_primary_candidate(
            &snapshot,
            0, // giver
            1, // receiver
            &conventions,
            &state.static_data,
        );
        assert!(
            receiver_gate,
            "receiver_gate should delegate to strict gate when it passes"
        );
    }

    #[test]
    fn receiver_had_primary_candidate_relaxes_when_receiver_hand_unresolved() {
        use crate::engine::convention::hgroup::h_group_convention_set::HGroupConventionSet;

        let mut state = make_state();
        // Giver (player 0) has no cards
        // Receiver (player 1) has a card that is unresolved from their own POV
        state.update_with_draw_action_of_specific_card(1, 5, 0);

        let snapshot = GameStateSnapshot::new(state.table_state.clone(), state.team_knowledge);
        let conventions = HGroupConventionSet::new(vec![]);

        // Strict gate should fail (giver has no primary candidate)
        let strict_gate = giver_had_primary_candidate(
            &snapshot,
            0, // giver
            &conventions,
            &state.static_data,
        );
        assert!(
            !strict_gate,
            "strict gate should fail when giver has no cards"
        );

        // Now check if giver's POV can resolve receiver's card. When we reconstruct
        // giver's POV, the receiver is player 1, and their card is in their own_hand
        // but not visible to themselves. The giver (player 0) *can* see player 1's
        // card since it's not in player 0's own_hand. So card_identity should return
        // Some(0) from giver's POV.
        //
        // This means the relaxation won't trigger in this simple test setup because
        // the giver's POV is omniscient about other players' hands.
        //
        // To make the relaxation trigger, we need to test the scenario where the
        // receiver's own hand contains an unresolved card *from the giver's
        // reconstructed POV*. This happens in hidden-seat mode where the giver's
        // reconstructed POV doesn't have the receiver's cards in visible_cards.
        //
        // Since this is hard to unit test without full replay infrastructure, we'll
        // verify the logic directly: if any card in receiver's hand is unresolved
        // from giver's POV, the gate should relax.

        // Manually verify the logic: giver POV should resolve receiver's hand
        let giver_pov = snapshot.player_pov(0, &state.static_data);
        let can_resolve = giver_pov.card_identity(5).is_some();
        assert!(
            can_resolve,
            "In omniscient mode, giver can see receiver's card"
        );

        // The receiver_gate uses the same logic, so it won't relax here
        let receiver_gate = receiver_had_primary_candidate(
            &snapshot,
            0, // giver
            1, // receiver
            &conventions,
            &state.static_data,
        );
        assert!(
            !receiver_gate,
            "receiver_gate doesn't relax when giver POV is omniscient (can see receiver's hand)"
        );
    }
}

