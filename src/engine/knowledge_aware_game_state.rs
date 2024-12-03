use crate::engine::convention::convention_set::ConventionSet;
use crate::engine::game_state_snapshot::GameStateSnapshot;
use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
use crate::engine::knowledge::player_pov_snapshot::PlayerPOVSnapshot;
use crate::engine::knowledge::team_knowledge::TeamKnowledge;
use crate::game::MAX_HAND_SIZE;
use crate::game::action::game_action::GameAction;
use crate::game::card::{CardDeckIndex, VariantCardId};
use crate::game::clue::Clue;
use crate::game::state::table_state::TableState;
use crate::game::static_game_data::StaticGameData;
use smallvec::SmallVec;

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
}

impl KnowledgeAwareGameState {
    pub fn new(static_data: StaticGameData) -> Self {
        let table_state = TableState::new(&static_data);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        KnowledgeAwareGameState {
            table_state,
            team_knowledge,
            static_data,
            next_deck_index: 0,
            history: Vec::new(),
        }
    }

    /// Construct from an existing table state and team knowledge (e.g. for search).
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
        }
    }

    /// Get the current turn number (sequential turn counter from table state).
    pub fn current_turn(&self) -> usize {
        self.table_state.current_turn
    }

    /// Get a read-only view of the game from the specified player's perspective.
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
    pub fn apply(&mut self, action: &GameAction, convention_set: &dyn ConventionSet) {
        match action {
            GameAction::Play {
                card_deck_index, ..
            } => self.apply_play(*card_deck_index, convention_set),
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
    }

    fn apply_play(&mut self, card_deck_index: CardDeckIndex, convention_set: &dyn ConventionSet) {
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
        let matched_tech = convention_set
            .techs()
            .iter()
            .find(|tech| tech.matches_action(&action, &self.history, &pov));
        let updates: Vec<_> = matched_tech
            .map(|tech| {
                tracing::debug!(target: "eel::apply", giver = p, action = ?action, "tech_matched");
                tech.knowledge_updates(&action, &self.history, &pov)
            })
            .unwrap_or_default();

        // Resolve the played card's identity so we can advance the stacks and give subsequent
        // players accurate playability information. Two cases where identity can be inferred:
        //
        // 1. Not clue-touched: inferred_identities (set by convention or scenario empathy) may
        //    have fully resolved the card to a single identity.
        // 2. Clue-touched + Signal::Play: the player knows the card is playable; intersecting the
        //    clue-narrowed empathy with the current playable set resolves it when exactly one
        //    playable identity matches (standard H-Group play-clue inference).
        //
        // Everything else falls back to the hidden-information path (card removed without
        // advancing stacks), since guessing wrong would corrupt the search state.
        let known_id = {
            let knowledge = self.team_knowledge.player(p);
            let has_play_signal = knowledge.signals[card_deck_index as usize].iter().any(|s| {
                matches!(
                    s,
                    crate::engine::convention::hgroup::signal::Signal::Play { .. }
                )
            });
            let combined = knowledge.combined_possible_identities(
                card_deck_index,
                &self.table_state,
                &self.static_data.variant,
            );
            // Primary: use the identity directly if fully resolved (covers both non-touched
            // cards with inferred_identities and clue-touched cards narrowed to a single card
            // by NarrowPossibilities from a play clue).
            // Fallback: for play signals whose empathy spans multiple candidates, intersect
            // with the current playable set (standard H-Group inference: "I know it's playable").
            combined.known_card_id().or_else(|| {
                if has_play_signal {
                    let playable = self.table_state.playable_cards(&self.static_data);
                    combined.narrow(playable).and_then(|e| e.known_card_id())
                } else {
                    None
                }
            })
        };
        if let Some(card_id) = known_id {
            self.table_state.update_with_play_action_of_specific_card(
                card_deck_index,
                card_id,
                &self.static_data,
            );
        } else {
            self.table_state.update_with_play_action(card_deck_index);
        }
        self.remove_card_from_own_hand(p, card_deck_index);
        self.update_with_unkown_card_draw(p);

        let num_players = self.static_data.number_of_players as usize;
        for target in (0..num_players).filter(|&t| t != p) {
            let own_hand = self.team_knowledge.player(target).own_hand;
            let filtered: Vec<_> = updates
                .iter()
                .filter(|u| {
                    use crate::engine::knowledge::knowledge_update::KnowledgeUpdate;
                    let idx = match u {
                        KnowledgeUpdate::NarrowPossibilities {
                            card_deck_index, ..
                        }
                        | KnowledgeUpdate::AddSignal {
                            card_deck_index, ..
                        } => *card_deck_index,
                    };
                    own_hand & (1 << idx) != 0
                })
                .cloned()
                .collect();
            if !filtered.is_empty() {
                tracing::debug!(target: "eel::apply", target, updates = ?filtered, "knowledge_updated");
                self.team_knowledge
                    .player_mut(target)
                    .apply_updates(&filtered, &self.static_data.variant);
            }
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
        let pre_clue_giver_pov = pre_clue_snapshot.player_pov(giver, &self.static_data);
        let matched_tech = convention_set
            .techs()
            .iter()
            .find(|tech| tech.matches_action(action, &self.history, &pre_clue_giver_pov));
        let Some(tech) = matched_tech else { return };
        tracing::debug!(target: "eel::apply", giver, action = ?action, "tech_matched");
        let receiver_pov = pre_clue_snapshot.player_pov(receiver, &self.static_data);
        let updates = tech.knowledge_updates(action, &self.history, &receiver_pov);
        self.team_knowledge
            .player_mut(receiver)
            .apply_updates(&updates, &self.static_data.variant);
        // Also update non-receiver players (e.g. prompted/finessed player in
        // simple_prompt/finesse Case 1). Only apply updates for cards in their own hand.
        let num_players = self.static_data.number_of_players as usize;
        for target in (0..num_players).filter(|&t| t != receiver) {
            // Build a temporary table state with player_on_turn_index set to `target` so that
            // tech.knowledge_updates can identify who the current observer is without mutating
            // the shared state (which would corrupt state if a panic or early return occurred).
            let mut target_table_state = self.table_state.clone();
            target_table_state.player_on_turn_index = target;
            let target_pov = LightweightPlayerPOV::new(
                target,
                self.team_knowledge.player(giver),
                &self.team_knowledge,
                &target_table_state,
                &self.static_data,
            );
            let updates = tech.knowledge_updates(action, &self.history, &target_pov);
            // Only apply updates for cards in target's own hand to avoid incorrectly
            // narrowing knowledge of cards in other players' hands.
            let own_hand = self.team_knowledge.player(target).own_hand;
            let filtered: Vec<_> = updates
                .into_iter()
                .filter(|u| {
                    use crate::engine::knowledge::knowledge_update::KnowledgeUpdate;
                    let idx = match u {
                        KnowledgeUpdate::NarrowPossibilities {
                            card_deck_index, ..
                        }
                        | KnowledgeUpdate::AddSignal {
                            card_deck_index, ..
                        } => *card_deck_index,
                    };
                    own_hand & (1 << idx) != 0
                })
                .collect();
            self.team_knowledge
                .player_mut(target)
                .apply_updates(&filtered, &self.static_data.variant);
            if !filtered.is_empty() {
                tracing::debug!(target: "eel::apply", target, updates = ?filtered, "knowledge_updated");
            }
        }
    }

    /// Advance `player_on_turn_index` to the next player.
    pub fn advance_turn(&mut self) {
        let num_players = self.static_data.number_of_players as usize;
        self.table_state.player_on_turn_index =
            (self.table_state.player_on_turn_index + 1) % num_players;
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

    pub fn table_state(&self) -> &TableState {
        &self.table_state
    }

    pub fn static_data(&self) -> &StaticGameData {
        &self.static_data
    }

    pub fn team_knowledge(&self) -> &TeamKnowledge {
        &self.team_knowledge
    }

    /// Capture the current board state and team knowledge as an owned snapshot.
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
    pub fn pov_at_turn(&self, turn: usize, player_index: usize) -> Option<PlayerPOVSnapshot> {
        let snapshot = self.history.get(turn)?.clone();
        if player_index >= self.static_data.number_of_players as usize {
            return None;
        }
        Some(PlayerPOVSnapshot::new(player_index, snapshot))
    }

    /// The number of snapshots recorded so far.
    pub fn history_len(&self) -> usize {
        self.history.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::card::Empathy;
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
            Empathy::all(&variant).as_bits(),
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
    fn player_pov_can_be_created_for_any_player() {
        let mut state = make_state();
        state.update_with_draw_action_of_specific_card(0, 7, 2);

        // Just verify player_pov builds without panic for each player.
        let _ = state.player_pov(0);
        let _ = state.player_pov(1);
        let _ = state.player_pov(2);
    }
}
