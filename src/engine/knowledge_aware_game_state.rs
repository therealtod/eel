use crate::engine::convention::convention_set::ConventionSet;
use crate::engine::convention::convention_tech::ConventionTech;
use crate::engine::game_state_snapshot::GameStateSnapshot;
use crate::engine::knowledge::knowledge_update::{Hypothesis, HypothesisId};
use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::engine::knowledge::player_pov_snapshot::PlayerPOVSnapshot;
use crate::engine::knowledge::team_knowledge::TeamKnowledge;
use crate::game::MAX_HAND_SIZE;
use crate::game::action::game_action::GameAction;
use crate::game::card::{CardDeckIndex, VariantCardId};
use crate::game::clue::Clue;
use crate::game::state::table_state::TableState;
use crate::game::static_game_data::StaticGameData;
use crate::game::variant::Variant;
use smallvec::SmallVec;

/// Collect hypotheses for `action` from `observer_pov`, respecting interpretation priority.
///
/// Collects up to two priority tiers (primary = highest matching, fallback = next highest).
/// Each returned entry is `(tier, hypothesis)` where `tier 0` = primary and `tier 1` = fallback.
/// Techs must be pre-sorted by `interpretation_priority` (ascending) —
/// `HGroupConventionSet::new` guarantees this. Empty hypotheses are dropped.
pub fn collect_hypotheses(
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
        let hyp = tech.knowledge_updates(action, history, observer_pov);
        if hyp.is_empty() {
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
        result.push((tier, hyp));
    }
    result
}

/// A [TableState] with associated player knowledge and convention awareness.
///
/// This is the main integration point for the engine: it wraps a [TableState] with a
/// [TeamKnowledge], keeping both in sync as actions are applied.
///
/// Two variants of each mutating method are provided:
/// - `*_of_specific_card`: used when the card identity is known (spectator / replay mode).
/// - without suffix: used when the identity is unknown (alpha-beta search over hidden state).
///
/// Call [`record_snapshot`](Self::record_snapshot) before each action to build up a turn
/// history. Use [`pov_at_turn`](Self::pov_at_turn) to retrieve any player's POV as it looked
/// at that moment — useful for retrospective analysis of why a player chose a given action.
#[derive(Clone)]
pub struct KnowledgeAwareGameState {
    pub table_state: TableState,
    pub team_knowledge: TeamKnowledge,
    static_data: StaticGameData,
    /// The deck index that will be assigned to the next synthesized draw.
    /// Initialized to `MAX_CARDS_IN_DECK - deck.current_size`.
    pub next_deck_index: u8,
    /// Per-turn snapshots recorded by [`record_snapshot`](Self::record_snapshot).
    /// Index `i` holds the state *before* the action taken on turn `i`.
    history: Vec<GameStateSnapshot>,
    /// Monotonic counter for unique hypothesis ids.
    next_hypothesis_id: HypothesisId,
    /// Successful plays whose identity was ambiguous (known-playable but multiple
    /// candidate identities). Tracked here — not in `TableState` — because the
    /// abstraction is engine/search-only: the score reflects that the play succeeded
    /// without committing to a specific stack, so future ply reasoning about
    /// playability and criticality of other ranks stays honest.
    phantom_plays: u8,
}

impl KnowledgeAwareGameState {
    #[must_use]
    pub fn new(static_data: StaticGameData) -> Self {
        let table_state = TableState::new(&static_data);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        KnowledgeAwareGameState {
            table_state,
            team_knowledge,
            static_data,
            next_deck_index: 0,
            history: Vec::new(),
            next_hypothesis_id: 0,
            phantom_plays: 0,
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
            static_data,
            next_deck_index,
            history: Vec::new(),
            next_hypothesis_id: 0,
            phantom_plays: 0,
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
        self.table_state.update_with_play_action_of_specific_card(
            card_deck_index,
            card_id,
            &self.static_data,
        );
        self.remove_card_from_own_hand(player_index, card_deck_index);
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

    /// Apply a `GameAction` (hidden-information flavour) and propagate convention knowledge.
    /// Does NOT advance the turn; call `advance_turn()` separately.
    ///
    /// For clue actions the `turn` field is set to `self.history_len() - 1` so the action
    /// permanently records which snapshot in `history` captures the state before this clue.
    /// Call [`record_snapshot`](Self::record_snapshot) *before* `apply` so the snapshot is
    /// already in `history` when this assignment runs.
    ///
    /// The search uses clone-and-recurse, so no undo token is needed.
    ///
    /// `truth` is the POV of the player who is reasoning about this action. For search,
    /// it is the ROOT searcher (held fixed across recursion), so play resolution can use
    /// the searcher's view of visible cards rather than the simulated active player's
    /// (possibly mistaken) empathy. For replay/tests with no specific thinker, callers
    /// should pass the active player's own POV — this preserves the legacy empathy-based
    /// resolution behavior.
    pub fn apply(
        &mut self,
        action: &GameAction,
        convention_set: &dyn ConventionSet,
        truth: &dyn PlayerPOV,
    ) {
        let actor = self.table_state.active_player_index();
        match action {
            GameAction::Play {
                card_deck_index, ..
            } => self.apply_play(*card_deck_index, convention_set, truth),
            GameAction::Discard {
                card_deck_index, ..
            } => self.apply_discard(*card_deck_index),
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
            }
            GameAction::Draw {
                card_deck_index,
                player_index,
            } => {
                self.table_state.update_with_draw_action(*card_deck_index);
                self.team_knowledge.player_mut(*player_index).own_hand |= 1u64 << *card_deck_index;
            }
        }

        // Resolve pending interpretations across all players keyed on `actor`'s action.
        // Draw actions never trigger a resolution.
        if !matches!(action, GameAction::Draw { .. }) {
            let num_players = self.static_data.number_of_players as usize;
            for p in 0..num_players {
                self.team_knowledge.player_mut(p).resolve_pending(
                    actor,
                    action,
                    &self.static_data.variant,
                );
            }
        }
    }

    fn apply_play(
        &mut self,
        card_deck_index: CardDeckIndex,
        convention_set: &dyn ConventionSet,
        truth: &dyn PlayerPOV,
    ) {
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
        let actor_hypotheses =
            collect_hypotheses(convention_set.techs(), &action, &self.history, &pov);
        if !actor_hypotheses.is_empty() {
            tracing::debug!(target: "eel::apply", giver = p, action = ?action, "tech_matched");
        }

        // Resolve the played card's identity (or its playable-but-ambiguous status) so that we
        // advance the engine's effective score and give subsequent plies accurate information.
        //
        // Three outcomes, in priority order:
        //
        // 1. Singleton identity: empathy (possibly narrowed by play signal ∩ playable) resolves to
        //    exactly one `card_id`. Use the full table-state path that advances the matching
        //    stack or records a strike. Covers fully-clued plays and BlindPlay/finesse plays
        //    that uniquely identify the card.
        //
        // 2. Known-playable but multi-id: the player has a `Signal::Play` and the empathy ∩
        //    playable intersection is non-empty (multiple playable candidates remain). The play
        //    is guaranteed to succeed in expectation, but we deliberately refuse to commit to
        //    a single stack — guessing would distort downstream playability/criticality
        //    reasoning ("I assumed r2 was played, so r3 is now playable"). Instead, increment
        //    `phantom_plays` so the effective score advances without revealing any identity.
        //
        // 3. Truly unknown: no signal, identity not narrowed. Fall back to the hidden-info path
        //    (card removed from hand, no score change). This is essentially unreachable for
        //    legitimate Play actions generated by `PlayKnownPlayable` / `BlindPlay`.
        let (known_id, has_play_signal) = {
            let knowledge = self.team_knowledge.player(p);
            let has_play_signal = knowledge.has_play_signal(card_deck_index);
            let combined = knowledge.combined_possible_identities(
                card_deck_index,
                &self.table_state,
                &self.static_data.variant,
            );
            let empathy_id = combined.known_card_id().or_else(|| {
                if has_play_signal {
                    let playable = self.table_state.playable_cards(&self.static_data);
                    combined.narrow(playable).and_then(|e| e.known_card_id())
                } else {
                    None
                }
            });
            // Truth override: prefer the thinker's view of the card when their truth
            // CONTRADICTS the active player's empathy — i.e. the empathy candidates do not
            // include the true identity. This catches:
            //  - empathy collapsed to a singleton, but truth says a different identity
            //    (duplicate-trash converging on a playable id);
            //  - empathy is ambiguous-but-all-playable, but truth is something outside
            //    that set (also typically trash) — the play should strike, not succeed.
            // When the truth IS one of the empathy candidates, fall back to empathy: the
            // active player's reasoning is consistent with the truth, and an ambiguous-
            // but-playable case must keep flowing into the phantom-play branch so the
            // search doesn't commit to a specific stack just because truth is visible.
            let truth_id = truth.card_identity(card_deck_index);
            let empathy_contains_truth = truth_id.is_some_and(|t| combined.as_bits() & (1 << t) != 0);
            let id = match (empathy_id, truth_id) {
                (_, Some(t)) if !empathy_contains_truth => Some(t),
                _ => empathy_id,
            };
            (id, has_play_signal)
        };
        if let Some(card_id) = known_id {
            self.table_state.update_with_play_action_of_specific_card(
                card_deck_index,
                card_id,
                &self.static_data,
            );
        } else if self.is_known_playable_play(p, card_deck_index, has_play_signal) {
            self.table_state.update_with_play_action(card_deck_index);
            self.add_phantom_play();
        } else {
            self.table_state.update_with_play_action(card_deck_index);
        }
        self.remove_card_from_own_hand(p, card_deck_index);
        self.update_with_unkown_card_draw(p);

        let num_players = self.static_data.number_of_players as usize;
        let cohort_id = self.next_hypothesis_id;
        self.next_hypothesis_id += 1;
        for target in (0..num_players).filter(|&t| t != p) {
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

    fn apply_discard(&mut self, card_deck_index: CardDeckIndex) {
        let p = self.table_state.active_player_index();
        let num_players = self.static_data.number_of_players as usize;
        // Prefer a spectator's inferred knowledge so that cards with known identities in other players'
        // hands are correctly identified as critical. Fall back to global deck empathy.
        let empathy = (0..num_players)
            .filter(|&obs| obs != p)
            .map(|obs| {
                let pk = self.team_knowledge.player(obs);
                pk.combined_possible_identities(
                    card_deck_index,
                    &self.table_state,
                    &self.static_data.variant,
                )
            })
            .find(|e| e.is_exactly_known())
            .unwrap_or_else(|| self.table_state.deck.get_global_empathy(card_deck_index));
        // Use add_card_with_id for the last copy so critical_in_discard scoring fires correctly.
        // For non-critical cards use the generic path to avoid spuriously inflating
        // critical_cards_in_hand for cards that only become critical during this search branch.
        let is_last_copy = empathy.known_card_id().is_some_and(|card_id| {
            let total = self.static_data.variant.card_copies_count_by_id[card_id];
            let discarded = self.table_state.discard_pile.copies_of(card_id);
            total > 0 && discarded == total - 1
        });
        if is_last_copy {
            let card_id = empathy.known_card_id().unwrap();
            self.table_state.hands[p].remove_card(card_deck_index);
            self.table_state.discard_pile.add_card_with_id(card_id);
            let bonus_tokens = self.static_data.variant.bonus_half_clue_tokens_for_discard;
            self.table_state.clue_token_bank.add_tokens(bonus_tokens);
        } else {
            self.table_state
                .update_with_discard_action(card_deck_index, &self.static_data);
        }
        self.remove_card_from_own_hand(p, card_deck_index);
        self.update_with_unkown_card_draw(p);
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
        let pre_clue_snapshot = self.snapshot();
        self.table_state.update_with_clue_action(
            touched_card_deck_indexes.clone(),
            clue.clone(),
            receiver,
            &self.static_data,
        );

        // Raw clue narrowing on the receiver's hand: touched cards must match the clue mask,
        // untouched cards must not. These are hard facts derived purely from the public clue;
        // applying them first ensures that softer convention inferences (good-touch, tech
        // hypotheses) can never widen empathy past what the literal clue establishes.
        let clue_mask = self.static_data.variant.empathy_for_clue(clue).as_bits();
        let hand_slots: SmallVec<[CardDeckIndex; MAX_HAND_SIZE]> = self.table_state.hands
            [receiver]
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

        // Convention-wide baseline narrowings on the receiver (e.g. H-Group good-touch
        // principle: every touched card is assumed eventually useful). Applied as
        // unconditional baseline so it intersects with per-tech cohort hypotheses.
        //
        // Per-card exception: if a touched card's post-raw empathy is already a subset of
        // non-needed identities (every candidate is trash by public information), the
        // holder won't misplay it without GTP help. Skip the GTP narrowing for that card
        // so we don't smuggle in fake "still-useful" plausibility on top of trash empathy.
        let still_needed = crate::engine::convention::hgroup::h_group_core::still_needed_cards_mask(
            &self.table_state,
            &self.static_data,
        );
        for (idx, mask) in convention_set.clue_receiver_baseline(
            clue,
            touched_card_deck_indexes,
            receiver,
            &self.table_state,
            &self.static_data,
        ) {
            let current_empathy = self.team_knowledge.player(receiver).inferred_identities
                [idx as usize]
                .map(|m| m.as_bits())
                .unwrap_or(u64::MAX);
            if current_empathy & still_needed == 0 {
                continue;
            }
            self.team_knowledge.player_mut(receiver).narrow_inferred(
                idx,
                mask,
                &self.static_data.variant,
            );
        }

        let cohort_id = self.next_hypothesis_id;
        self.next_hypothesis_id += 1;
        let num_players = self.static_data.number_of_players as usize;
        let techs = convention_set.techs();

        // In normal gameplay `record_snapshot` is called before each action, so
        // `self.history[turn]` holds the pre-clue state that `knowledge_updates` need.
        // In search no history is recorded; synthesise a slice padded up to `action.turn`
        // so `history.get(action.turn)` resolves to `pre_clue_snapshot`. Padding (rather
        // than a single-entry slice) is required because search advances `current_turn`,
        // so simulated actions past the first ply carry `turn > 0`.
        let local_history;
        let knowledge_history: &[GameStateSnapshot] = if !self.history.is_empty() {
            &self.history
        } else {
            let action_turn = match action {
                GameAction::Clue { turn, .. } => *turn,
                _ => 0,
            };
            local_history = vec![pre_clue_snapshot.clone(); action_turn + 1];
            &local_history
        };

        // Receiver: collect all matching techs' hypotheses from the receiver's own POV.
        let receiver_pov = pre_clue_snapshot.player_pov(receiver, &self.static_data);
        let receiver_hypotheses =
            collect_hypotheses(techs, action, knowledge_history, &receiver_pov);
        if !receiver_hypotheses.is_empty() {
            tracing::debug!(target: "eel::apply", giver, action = ?action, hypotheses = receiver_hypotheses.len(), "receiver_hypotheses");
        }
        self.team_knowledge.player_mut(receiver).apply_cohort(
            cohort_id,
            receiver_hypotheses,
            &mut self.next_hypothesis_id,
            &self.static_data.variant,
        );

        // Non-receivers: each observer evaluates all techs from their own POV. Receiver-targeted
        // updates are filtered down to the observer's own hand (other-hand updates are redundant
        // since they can see those hands directly via visible_cards).
        for target in (0..num_players).filter(|&t| t != receiver) {
            let mut target_table_state = self.table_state.clone();
            target_table_state.active_player_index = target;
            let target_pov = LightweightPlayerPOV::new(
                target,
                self.team_knowledge.player(giver),
                &self.team_knowledge,
                &target_table_state,
                &self.static_data,
            );
            let raw = collect_hypotheses(techs, action, knowledge_history, &target_pov);
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

    /// Number of successful plays whose stack assignment was deferred (engine-only).
    /// Always zero outside the search.
    #[must_use]
    pub fn phantom_plays(&self) -> u8 {
        self.phantom_plays
    }

    /// Effective game score: real stack progress plus successful-but-unattributed plays.
    #[must_use]
    pub fn score(&self, variant: &Variant) -> u8 {
        self.table_state.score(variant) + self.phantom_plays
    }

    /// Pace adjusted for phantom plays: phantoms count as score progress.
    #[must_use]
    pub fn pace(&self) -> i32 {
        self.table_state.pace(&self.static_data) + self.phantom_plays as i32
    }

    /// Required efficiency adjusted for phantom plays.
    ///
    /// Mirrors [`TableState::required_efficiency`] but treats phantom plays as already-played
    /// cards: they reduce `still_to_play` and increase `spare_turns` (because the card has
    /// already left a hand). The "live setups" term is taken from `TableState` unchanged.
    #[must_use]
    pub fn required_efficiency(&self) -> f32 {
        let variant = &self.static_data.variant;
        let max_score = (variant.number_of_suits * variant.stacks_size) as i32;
        let num_players = self.static_data.number_of_players as usize;
        let hand_cards: i32 = self.table_state.hands[..num_players]
            .iter()
            .map(|h| h.cards().len() as i32)
            .sum();
        let remaining = self.table_state.deck.current_size as i32 + hand_cards;
        let still_to_play = max_score - self.score(variant) as i32;
        let spare_turns = (remaining - still_to_play).max(0);
        if still_to_play <= 0 {
            return 0.0;
        }
        if spare_turns == 0 {
            return f32::INFINITY;
        }
        let all_hand_bits = self.table_state.all_hand_bits;
        let live_setups =
            (self.table_state.clue_touched_cards & all_hand_bits).count_ones() as i32;
        (still_to_play - live_setups).max(0) as f32 / spare_turns as f32
    }

    /// True once the team has reached the theoretical max score (counting phantom plays)
    /// or struck out. Used by the search to detect terminal nodes without leaking the
    /// phantom abstraction into `TableState`.
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        let max_score = self.static_data.variant.number_of_suits * self.static_data.variant.stacks_size;
        self.table_state.strike_tokens >= 3 || self.score(&self.static_data.variant) >= max_score
    }

    /// Record that a successful play has happened without committing to a specific stack.
    /// The caller is responsible for also removing the played card from the relevant
    /// table-state hand and team-knowledge own-hand bitmap (see [`apply_play`](Self::apply_play)).
    fn add_phantom_play(&mut self) {
        self.phantom_plays = self.phantom_plays.saturating_add(1);
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
    fn is_known_playable_play(
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

    /// Push a snapshot of the current state onto the history.
    ///
    /// Call this *before* applying the action for a given turn so that
    /// `history[t]` reflects the state each player saw when deciding on turn `t`.
    pub fn record_snapshot(&mut self) {
        self.history.push(self.snapshot());
    }

    /// Retrieve the POV of `player_index` as it looked at the start of turn `turn`.
    ///
    /// Returns `None` if `turn` is out of range (no snapshot was recorded for it)
    /// or `player_index` is invalid.
    ///
    /// Call [`PlayerPOVSnapshot::as_pov`] with [`Self::static_data`] to materialise a
    /// [`LightweightPlayerPOV`] from the returned snapshot.
    #[must_use]
    pub fn pov_at_turn(&self, turn: usize, player_index: usize) -> Option<PlayerPOVSnapshot> {
        let snapshot = self.history.get(turn)?.clone();
        if player_index >= self.static_data.number_of_players as usize {
            return None;
        }
        Some(PlayerPOVSnapshot::new(player_index, snapshot))
    }

    /// The number of snapshots recorded so far.
    #[must_use]
    pub fn history_len(&self) -> usize {
        self.history.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::card::CardIdentityMask;
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
        let pre_score = state.score(&state.static_data.variant);
        assert_eq!(state.phantom_plays(), 0);

        let conventions = HGroupConventionSet::new(vec![]);
        let play_action = GameAction::Play {
            player_index: 0,
            card_deck_index: 0,
            turn: 0,
        };
        let knowledge_clone = state.team_knowledge.player(0).clone();
        let team_clone = state.team_knowledge.clone();
        let table_clone = state.table_state.clone();
        let static_clone = state.static_data.clone();
        let truth = LightweightPlayerPOV::new(
            0,
            &knowledge_clone,
            &team_clone,
            &table_clone,
            &static_clone,
        );
        state.apply(&play_action, &conventions, &truth);

        assert_eq!(
            state.phantom_plays(),
            1,
            "phantom_plays should increment for an ambiguous-but-playable play"
        );
        assert_eq!(
            state.table_state.score(&state.static_data.variant),
            pre_stack_total,
            "no concrete stack should advance for a phantom play"
        );
        assert_eq!(
            state.score(&state.static_data.variant),
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
        let truth = LightweightPlayerPOV::new(
            0,
            &knowledge_clone,
            &team_clone,
            &table_clone,
            &static_clone,
        );
        state.apply(
            &GameAction::Play {
                player_index: 0,
                card_deck_index: 0,
                turn: 0,
            },
            &conventions,
            &truth,
        );

        assert_eq!(state.phantom_plays(), 0);
        assert_eq!(state.table_state.score(&state.static_data.variant), 1);
        assert_eq!(state.score(&state.static_data.variant), 1);
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
}
