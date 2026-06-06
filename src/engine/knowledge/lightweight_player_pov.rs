use crate::engine::convention::hgroup::h_group_core::clues_for_player_with_focus;
use crate::engine::convention::hgroup::signal::Signal;
use crate::engine::knowledge::player_knowledge::PlayerKnowledge;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::engine::knowledge::team_knowledge::TeamKnowledge;
use crate::game::MAX_CLUE_TOKEN_COUNT;
use crate::game::action::game_action::GameAction;
use crate::game::card::{CardDeckIndex, CardIdentityMask, DeckCardsBitField, VariantCardId};
use crate::game::state::PlayerIndex;
use crate::game::state::table_state::TableState;
use crate::game::static_game_data::StaticGameData;

/// Lightweight, read-only view that combines shared game state with player-specific knowledge.
///
/// Created on-the-fly when convention techs need to evaluate the game from a player's perspective.
/// Does **not** own any data — borrows everything from the single `TableState` and
/// per-player `PlayerKnowledge`.
pub struct LightweightPlayerPOV<'a> {
    player_index: usize,
    knowledge: &'a PlayerKnowledge,
    team_knowledge: &'a TeamKnowledge,
    table_state: &'a TableState,
    static_data: &'a StaticGameData,
    /// Effective sight set: which cards this POV can actually observe. Defaults to
    /// `knowledge.visible_cards`. During search, [`teammate_pov`](Self::teammate_pov)
    /// produces a POV with an intersected `visible_cards` so a teammate cannot peek
    /// at hands the root observer cannot see.
    visible_cards: DeckCardsBitField,
}

impl<'a> LightweightPlayerPOV<'a> {
    #[must_use]
    pub fn new(
        player_index: usize,
        knowledge: &'a PlayerKnowledge,
        team_knowledge: &'a TeamKnowledge,
        table_state: &'a TableState,
        static_data: &'a StaticGameData,
    ) -> Self {
        LightweightPlayerPOV {
            player_index,
            knowledge,
            team_knowledge,
            table_state,
            static_data,
            visible_cards: knowledge.visible_cards,
        }
    }

    /// Construct a POV with an explicit effective sight set, overriding
    /// `knowledge.visible_cards`. Used by [`teammate_pov`](Self::teammate_pov).
    #[must_use]
    pub fn with_visible_cards(
        player_index: usize,
        knowledge: &'a PlayerKnowledge,
        team_knowledge: &'a TeamKnowledge,
        table_state: &'a TableState,
        static_data: &'a StaticGameData,
        visible_cards: DeckCardsBitField,
    ) -> Self {
        LightweightPlayerPOV {
            player_index,
            knowledge,
            team_knowledge,
            table_state,
            static_data,
            visible_cards,
        }
    }

    /// Reconstruct `target`'s POV from this observer's POV, intersecting effective
    /// sight so the new POV cannot see anything the observer can't see either.
    ///
    /// Used during search: when Bob's root POV simulates Alice's turn, Alice's
    /// search-time view of Bob's hand collapses (Bob can't see his own hand, so
    /// the intersection drops those bits). Alice's reasoning falls back to public
    /// clue/convention knowledge only — no omniscient peek.
    #[must_use]
    pub fn teammate_pov(&self, target: PlayerIndex) -> LightweightPlayerPOV<'a> {
        let target_pk = self.team_knowledge.player(target);
        let visible_cards = self.visible_cards & target_pk.visible_cards;
        LightweightPlayerPOV::with_visible_cards(
            target,
            target_pk,
            self.team_knowledge,
            self.table_state,
            self.static_data,
            visible_cards,
        )
    }

    /// Effective sight set for this POV.
    #[must_use]
    pub fn effective_visible_cards(&self) -> DeckCardsBitField {
        self.visible_cards
    }
}

impl PlayerPOV for LightweightPlayerPOV<'_> {
    fn player_index(&self) -> PlayerIndex {
        self.player_index
    }

    fn away_value(&self, card_id: VariantCardId) -> Option<u8> {
        if self
            .table_state
            .playing_stacks
            .contains_card_with_id(card_id)
        {
            return None;
        }
        let mut away_value: u8 = 0;
        let mut current_card_id = card_id;
        while let Some(prereq_id) = self.static_data.variant.prerequisite(current_card_id) {
            if self
                .table_state
                .playing_stacks
                .contains_card_with_id(prereq_id)
            {
                break;
            }
            away_value += 1;
            current_card_id = prereq_id;
        }
        Some(away_value)
    }

    fn card_identity(&self, card_deck_index: CardDeckIndex) -> Option<VariantCardId> {
        self.inferred_identities(card_deck_index).known_card_id()
    }

    #[allow(clippy::cast_possible_truncation)]
    fn own_playable_cards(&self) -> DeckCardsBitField {
        // Precompute the playable mask once, then reuse the canonical per-card predicate.
        let playable_mask = self.table_state.playable_cards(self.static_data);
        let mut result: DeckCardsBitField = 0;
        let mut hand_mask = self.knowledge.own_hand;
        while hand_mask != 0 {
            let card_deck_index = hand_mask.trailing_zeros() as CardDeckIndex;
            if self.is_playable_with_playable_mask(card_deck_index, playable_mask) {
                result |= 1 << card_deck_index;
            }
            hand_mask &= hand_mask - 1;
        }
        result
    }

    fn is_clue_touched(&self, card_deck_index: CardDeckIndex) -> bool {
        self.table_state.clue_touched_cards & (1 << card_deck_index) != 0
    }

    fn is_signal_touched(&self, card_deck_index: CardDeckIndex) -> bool {
        let num_players = self.static_data.number_of_players as usize;
        (0..num_players).any(|p| {
            let pk = self.team_knowledge.player(p);
            pk.own_hand & (1 << card_deck_index) != 0 && pk.has_play_signal(card_deck_index)
        })
    }

    fn is_identity_known_to_holder(&self, card_deck_index: CardDeckIndex) -> bool {
        let num_players = self.static_data.number_of_players as usize;
        (0..num_players).any(|p| {
            let pk = self.team_knowledge.player(p);
            if pk.own_hand & (1 << card_deck_index) == 0 {
                return false;
            }
            // Direct sight or singleton inference.
            if pk.knows_identity(card_deck_index) {
                return true;
            }
            // Known via a play signal (e.g. finesse blind-play)
            if pk.signals[card_deck_index as usize]
                .iter()
                .any(|s| matches!(s, Signal::Play { .. }))
            {
                return true;
            }
            // Known via combined inferred + observable narrowing collapsing to a singleton.
            self.as_player_pov(p)
                .inferred_identities(card_deck_index)
                .known_card_id()
                .is_some()
        })
    }

    fn is_critical(&self, card_deck_index: CardDeckIndex) -> bool {
        let Some(card_id) = self.card_identity(card_deck_index) else {
            return false;
        };
        self.is_critical_card_id(card_id)
    }

    fn is_critical_card_id(&self, variant_card_id: VariantCardId) -> bool {
        let total = self.static_data.variant.card_copies_count_by_id[variant_card_id];
        let discarded = self.table_state.discard_pile.copies_of(variant_card_id);
        total > 0 && discarded == total - 1
    }

    fn active_player_index(&self) -> PlayerIndex {
        self.table_state.active_player_index()
    }

    fn table_state(&self) -> &TableState {
        self.table_state
    }

    fn static_data(&self) -> &StaticGameData {
        self.static_data
    }

    fn team_knowledge(&self) -> &TeamKnowledge {
        self.team_knowledge
    }

    fn visible_cards(&self) -> DeckCardsBitField {
        self.visible_cards
    }

    fn is_known_trash(&self, card_deck_index: CardDeckIndex) -> bool {
        let possible_bits = self.inferred_identities(card_deck_index).as_bits();

        // Identities still needing to be played (not already on the stacks). The card is
        // known trash only when *every* such identity is also unreachable; a single live
        // candidate is enough to make discarding it unsafe.
        let played_cards = self.table_state.playing_stacks.as_bitfield();
        let mut live = possible_bits & !played_cards;
        if live == 0 {
            return true;
        }

        let variant = &self.static_data.variant;
        let discarded = &self.table_state.discard_pile;
        while live != 0 {
            let bit_pos = live.trailing_zeros() as usize;
            live &= live - 1;
            let card_id = bit_pos as VariantCardId;
            // An identity is reachable unless one of its prerequisites is fully discarded.
            let mut prereq = variant.prerequisite(card_id);
            let mut reachable = true;
            while let Some(prereq_id) = prereq {
                let total = variant.card_copies_count_by_id[prereq_id];
                let discarded_count = discarded.copies_of(prereq_id);
                if discarded_count == total {
                    reachable = false;
                    break;
                }
                prereq = variant.prerequisite(prereq_id);
            }
            // A live, reachable candidate means the card might still be needed.
            if reachable {
                return false;
            }
        }

        true
    }

    fn inferred_identities(&self, card_deck_index: CardDeckIndex) -> CardIdentityMask {
        self.knowledge.combined_possible_identities_with_visible(
            card_deck_index,
            self.visible_cards,
            self.table_state,
            &self.static_data.variant,
        )
    }

    #[allow(clippy::cast_possible_truncation)]
    fn valid_actions(&self) -> Vec<GameAction> {
        let player_index = self.player_index;
        let clue_tokens = self.table_state.clue_token_bank.whole_clue_tokens_count();
        let num_players = self.static_data.number_of_players as usize;
        let mut actions = Vec::new();

        let mut hand_mask = self.knowledge.own_hand;
        while hand_mask != 0 {
            let card_deck_index = hand_mask.trailing_zeros() as CardDeckIndex;
            actions.push(GameAction::Play {
                player_index,
                card_deck_index,
                turn: self.table_state.current_turn,
            });
            if clue_tokens < MAX_CLUE_TOKEN_COUNT {
                actions.push(GameAction::Discard {
                    player_index,
                    card_deck_index,
                    turn: self.table_state.current_turn,
                });
            }
            hand_mask &= hand_mask - 1;
        }

        if clue_tokens > 0 {
            for target in 0..num_players {
                if target == player_index {
                    continue;
                }
                for (clue_action, _focus) in clues_for_player_with_focus(target, self) {
                    actions.push(clue_action);
                }
            }
        }

        actions
    }

    fn gotten_cards(&self) -> CardIdentityMask {
        let num_players = self.static_data().number_of_players as usize;
        let mut bits: u64 = 0;
        for player_index in 0..num_players {
            let player_hand = &self.table_state.hands[player_index];
            for &card_deck_index in player_hand.cards() {
                if let Some(card_identity) = self.card_identity(card_deck_index)
                    && self.is_touched(card_deck_index) {
                        bits |= 1 << card_identity;
                    }
            }
        }
        CardIdentityMask::from_bits(bits)
    }

    fn is_gotten(&self, variant_card_id: VariantCardId) -> bool {
        self.gotten_cards().contains(variant_card_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::knowledge::player_knowledge::knowledge_with_empathy;

    use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards::*;
    use crate::game::deck::unit_test_constants::novariant_constants::*;
    use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
        NOVAR_5_PLAYERS_STATIC_GAME_DATA, initial_five_players_table_state,
    };

    #[test]
    fn knows_that_a_fully_known_card_is_playable() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(0);
        table_state.update_with_draw_action(1);
        table_state.update_with_play_action_of_specific_card(
            0,
            R1.as_variant_card_id(),
            &static_data,
        );
        table_state.update_with_play_action_of_specific_card(
            1,
            R2.as_variant_card_id(),
            &static_data,
        );

        let knowledge = knowledge_with_empathy(42, R3_MASK);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(pov.is_playable(42));
    }

    #[test]
    fn knows_that_a_fully_known_card_is_not_playable() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(0);
        table_state.update_with_draw_action(1);
        table_state.update_with_play_action_of_specific_card(
            0,
            R1.as_variant_card_id(),
            &static_data,
        );
        table_state.update_with_play_action_of_specific_card(
            1,
            R2.as_variant_card_id(),
            &static_data,
        );

        let knowledge = knowledge_with_empathy(42, B3_MASK);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(!pov.is_playable(42));
    }

    #[test]
    fn knows_a_card_is_playable_because_all_possible_identities_are_playable() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(0);
        table_state.update_with_draw_action(1);
        table_state.update_with_draw_action(2);
        table_state.update_with_draw_action(3);
        table_state.update_with_play_action_of_specific_card(
            0,
            Y1.as_variant_card_id(),
            &static_data,
        );
        table_state.update_with_play_action_of_specific_card(
            1,
            B1.as_variant_card_id(),
            &static_data,
        );
        table_state.update_with_play_action_of_specific_card(
            2,
            B2.as_variant_card_id(),
            &static_data,
        );
        table_state.update_with_play_action_of_specific_card(
            3,
            R2.as_variant_card_id(),
            &static_data,
        );

        let knowledge = knowledge_with_empathy(42, R1_MASK | Y2_MASK | G1_MASK | B3_MASK | P1_MASK);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(pov.is_playable(42));
    }

    #[test]
    fn a_card_is_not_playable_because_not_all_possible_identities_are_playable() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(0);
        table_state.update_with_draw_action(1);
        table_state.update_with_draw_action(2);
        table_state.update_with_draw_action(3);
        table_state.update_with_play_action_of_specific_card(
            0,
            Y1.as_variant_card_id(),
            &static_data,
        );
        table_state.update_with_play_action_of_specific_card(
            1,
            B1.as_variant_card_id(),
            &static_data,
        );
        table_state.update_with_play_action_of_specific_card(
            2,
            B2.as_variant_card_id(),
            &static_data,
        );
        table_state.update_with_play_action_of_specific_card(
            3,
            R2.as_variant_card_id(),
            &static_data,
        );

        let knowledge = knowledge_with_empathy(
            42,
            R1_MASK | Y2_MASK | Y3_MASK | G1_MASK | B3_MASK | P1_MASK,
        );
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(!pov.is_playable(42));
    }

    #[test]
    fn knows_card_is_trash_when_copy_already_played() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(0);
        table_state.update_with_draw_action(1);
        table_state.update_with_play_action_of_specific_card(
            0,
            R1.as_variant_card_id(),
            &static_data,
        );

        let knowledge = knowledge_with_empathy(42, R1_MASK);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(pov.is_known_trash(42));
    }

    #[test]
    fn knows_card_is_trash_when_prerequisite_discarded() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(0);
        table_state.update_with_draw_action(1);
        table_state.update_with_draw_action(2);
        table_state.update_with_discard_action_of_specific_card(
            0,
            R1.as_variant_card_id(),
            &static_data,
        );
        table_state.update_with_discard_action_of_specific_card(
            1,
            R1.as_variant_card_id(),
            &static_data,
        );
        table_state.update_with_discard_action_of_specific_card(
            2,
            R1.as_variant_card_id(),
            &static_data,
        );

        let knowledge = knowledge_with_empathy(42, R2_MASK);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(pov.is_known_trash(42));
    }

    #[test]
    fn card_is_not_trash_when_prerequisite_still_available() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(0);
        table_state.update_with_draw_action(1);
        table_state.update_with_draw_action(2);
        table_state.update_with_discard_action_of_specific_card(
            0,
            R1.as_variant_card_id(),
            &static_data,
        );

        let knowledge = knowledge_with_empathy(42, R2_MASK);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(!pov.is_known_trash(42));
    }

    #[test]
    fn knows_card_is_trash_when_only_some_prerequisites_discarded_but_not_all() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(0);
        table_state.update_with_draw_action(1);
        table_state.update_with_draw_action(2);
        table_state.update_with_discard_action_of_specific_card(
            0,
            R1.as_variant_card_id(),
            &static_data,
        );
        table_state.update_with_discard_action_of_specific_card(
            1,
            R1.as_variant_card_id(),
            &static_data,
        );
        table_state.update_with_discard_action_of_specific_card(
            2,
            R1.as_variant_card_id(),
            &static_data,
        );

        let knowledge = knowledge_with_empathy(42, R2_MASK);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(pov.is_known_trash(42));
    }

    #[test]
    fn card_is_not_trash_when_playable() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(0);
        table_state.update_with_draw_action(1);

        let knowledge = knowledge_with_empathy(42, R1_MASK);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(!pov.is_known_trash(42));
    }

    #[test]
    fn is_known_trash_true_when_all_multi_bit_identities_are_played() {
        // Card 42 could be R1 or Y1 (two possible identities). Both are played to the stacks.
        // Since every possible identity is on the stack, the card is definitely useless.
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards::*;
        use crate::game::deck::unit_test_constants::novariant_constants::Y1_MASK;

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(0);
        table_state.update_with_draw_action(1);
        table_state.update_with_play_action_of_specific_card(
            0,
            R1.as_variant_card_id(),
            &static_data,
        );
        table_state.update_with_play_action_of_specific_card(
            1,
            Y1.as_variant_card_id(),
            &static_data,
        );

        let knowledge = knowledge_with_empathy(42, R1_MASK | Y1_MASK);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(pov.is_known_trash(42));
    }

    #[test]
    fn is_not_known_trash_when_one_multi_bit_identity_still_needed() {
        // Card 42 could be R1 (played) or B1 (not yet played). Because B1 is still needed the
        // card cannot be safely discarded — is_known_trash must return false.
        use crate::game::deck::unit_test_constants::novariant_constants::B1_MASK;
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards::*;

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(0);
        table_state.update_with_play_action_of_specific_card(
            0,
            R1.as_variant_card_id(),
            &static_data,
        );

        let knowledge = knowledge_with_empathy(42, R1_MASK | B1_MASK);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(!pov.is_known_trash(42));
    }

    #[test]
    fn is_not_known_trash_when_live_identity_coexists_with_dead_prerequisite_identity() {
        // Card 42 could be R2 (playable: R1 is on the stack) or B2 (unreachable: all three
        // B1 copies are discarded). Even though one candidate is dead, the other is live,
        // so the card is not known trash — discarding it could lose the needed R2.
        use crate::game::deck::unit_test_constants::novariant_constants::B2_MASK;
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards::*;
        use crate::game::deck::unit_test_constants::novariant_constants::R2_MASK;

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        for &idx in &[0u8, 1, 2, 3] {
            table_state.update_with_draw_action(idx);
        }
        // R1 played → R2 is now playable.
        table_state.update_with_play_action_of_specific_card(
            0,
            R1.as_variant_card_id(),
            &static_data,
        );
        // All three B1 copies discarded → B2 is unreachable.
        for player in 1..=3 {
            table_state.update_with_discard_action_of_specific_card(
                player,
                B1.as_variant_card_id(),
                &static_data,
            );
        }

        let knowledge = knowledge_with_empathy(42, R2_MASK | B2_MASK);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(!pov.is_known_trash(42));
    }

    #[test]
    fn is_known_trash_uses_observable_narrowing() {
        // Card 42 in player 0's hand is convention-narrowed to {R3, R4} (e.g. a "red"
        // play interpretation). R3 has been played and R4 would be playable. Using
        // only the combined empathy, the card looks like it could still be R4 (so it
        // is not yet known trash). However, P0 can directly see both R4 copies in
        // P1's hand, so observable narrowing rules out R4 — and the only remaining
        // candidate (R3) is already on the stack. The card is therefore actually
        // known trash.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        // Cards 0 and 1 are physically in P1's hand (both R4); cards 2, 3, 4 stand in
        // for the played R1/R2/R3 copies so the play actions resolve cleanly from the
        // active player's hand.
        for &(player, idx) in &[(1usize, 0u8), (1, 1), (0, 2), (0, 3), (0, 4)] {
            table_state.hands[player].add_card_to_slot_1(idx);
            table_state.all_hands_bitfield |= 1u64 << idx;
        }
        table_state.update_with_play_action_of_specific_card(
            2,
            R1.as_variant_card_id(),
            &static_data,
        );
        table_state.update_with_play_action_of_specific_card(
            3,
            R2.as_variant_card_id(),
            &static_data,
        );
        table_state.update_with_play_action_of_specific_card(
            4,
            R3.as_variant_card_id(),
            &static_data,
        );

        let mut knowledge = knowledge_with_empathy(42, R3_MASK | R4_MASK);
        knowledge.own_hand |= 1 << 42;
        table_state.hands[0].add_card_to_slot_1(42);
        table_state.all_hands_bitfield |= 1u64 << 42;
        knowledge.update_with_revealed_card(0, R4.as_variant_card_id());
        knowledge.update_with_revealed_card(1, R4.as_variant_card_id());
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).own_hand |= 1 << 42;
        team_knowledge.player_mut(1).own_hand |= (1 << 0) | (1 << 1);

        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(pov.is_known_trash(42));
    }

    #[test]
    fn gotten_cards_includes_touched_card_with_known_identity() {
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards::*;

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(0);
        table_state.clue_touched_cards |= 1 << 0;

        let knowledge = knowledge_with_empathy(0, R1_MASK);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).own_hand |= 1 << 0;

        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let gotten = pov.gotten_cards();
        assert!(gotten.contains(R1.as_variant_card_id()));
    }

    #[test]
    fn gotten_cards_includes_card_with_play_signal() {
        use crate::engine::convention::hgroup::signal::Signal;

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(0);

        let mut knowledge = knowledge_with_empathy(0, R1_MASK);
        knowledge.signals[0].push(Signal::Play {
            card_deck_index: 0,
            committed_identity: R1.as_variant_card_id(),
        });
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).own_hand |= 1 << 0;
        team_knowledge.player_mut(0).signals[0].push(Signal::Play {
            card_deck_index: 0,
            committed_identity: R1.as_variant_card_id(),
        });

        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let gotten = pov.gotten_cards();
        assert!(gotten.contains(R1.as_variant_card_id()));
    }

    #[test]
    fn gotten_cards_excludes_untouched_card_without_signal() {
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards::*;

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(0);

        let knowledge = knowledge_with_empathy(0, R1_MASK);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);

        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let gotten = pov.gotten_cards();
        assert!(!gotten.contains(R1.as_variant_card_id()));
    }

    #[test]
    fn gotten_cards_excludes_card_with_unknown_identity() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(0);
        table_state.clue_touched_cards |= 1 << 0;

        let knowledge = knowledge_with_empathy(0, R1_MASK | B1_MASK);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).own_hand |= 1 << 0;

        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let gotten = pov.gotten_cards();
        assert_eq!(gotten.as_bits(), 0);
    }

    /// Build a `PlayerKnowledge` for player 0 with several hand cards, each given an
    /// explicit empathy mask. `own_hand` is set to the union of the supplied indices.
    fn knowledge_with_hand(cards: &[(CardDeckIndex, u64)]) -> PlayerKnowledge {
        let mut k = PlayerKnowledge::new(0);
        for &(idx, mask) in cards {
            k.inferred_identities[idx as usize] = Some(CardIdentityMask::from_bits(mask));
            k.own_hand |= 1 << idx;
        }
        k
    }

    #[test]
    fn hand_is_locked_when_all_cards_touched_and_ambiguous() {
        // Empty stacks: R2/Y2/G2 are neither playable (their 1s aren't down) nor
        // trash, so each touched card is "stuck" — the hand is locked.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(0);
        table_state.update_with_draw_action(1);
        table_state.update_with_draw_action(2);
        table_state.clue_touched_cards |= (1 << 0) | (1 << 1) | (1 << 2);

        let knowledge = knowledge_with_hand(&[
            (0, R2_MASK | Y2_MASK),
            (1, G2_MASK | B2_MASK),
            (2, P2_MASK | R3_MASK),
        ]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).own_hand = knowledge.own_hand;

        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(pov.is_hand_locked());
    }

    #[test]
    fn hand_is_not_locked_when_a_card_is_untouched() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(0);
        table_state.update_with_draw_action(1);
        // Only card 0 is touched; card 1 is not.
        table_state.clue_touched_cards |= 1 << 0;

        let knowledge = knowledge_with_hand(&[(0, R2_MASK | Y2_MASK), (1, G2_MASK | B2_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).own_hand = knowledge.own_hand;

        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(!pov.is_hand_locked());
    }

    #[test]
    fn hand_is_not_locked_when_a_card_is_known_playable() {
        // Card 1 is the known-playable R1 (empty stacks), so the hand isn't locked.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(0);
        table_state.update_with_draw_action(1);
        table_state.clue_touched_cards |= (1 << 0) | (1 << 1);

        let knowledge = knowledge_with_hand(&[(0, R2_MASK | Y2_MASK), (1, R1_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).own_hand = knowledge.own_hand;

        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(!pov.is_hand_locked());
    }

    #[test]
    fn hand_is_locked_when_a_card_is_playable_but_holder_is_unaware() {
        // Card 0 could be R1 (playable on empty stacks) or R2 (not). Even though the
        // card might *physically* be the playable R1, the holder cannot tell — empathy
        // is ambiguous and there's no play signal — so it is not "known playable" and
        // the hand stays locked.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(0);
        table_state.update_with_draw_action(1);
        table_state.clue_touched_cards |= (1 << 0) | (1 << 1);

        let knowledge = knowledge_with_hand(&[(0, R1_MASK | R2_MASK), (1, G2_MASK | B2_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).own_hand = knowledge.own_hand;

        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(pov.is_hand_locked());
    }

    #[test]
    fn hand_is_not_locked_when_a_card_is_known_trash() {
        // R1 is played, so a known-R1 card (card 1) is trash → hand not locked.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(0);
        table_state.update_with_draw_action(1);
        table_state.update_with_play_action_of_specific_card(
            0,
            R1.as_variant_card_id(),
            &static_data,
        );
        table_state.clue_touched_cards |= (1 << 1) | (1 << 2);

        let knowledge = knowledge_with_hand(&[(1, R2_MASK | Y2_MASK), (2, R1_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).own_hand = knowledge.own_hand;

        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(!pov.is_hand_locked());
    }

    #[test]
    fn empty_hand_is_not_locked() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let table_state = initial_five_players_table_state();

        let knowledge = PlayerKnowledge::new(0);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);

        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(!pov.is_hand_locked());
    }

    #[test]
    fn gotten_cards_includes_cards_from_other_players() {
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards::*;

        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(0);
        table_state.update_with_draw_action(1);
        table_state.clue_touched_cards |= 1 << 1;

        let mut knowledge = knowledge_with_empathy(0, R1_MASK);
        knowledge.update_with_revealed_card(1, R2.as_variant_card_id());
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).own_hand |= 1 << 0;
        team_knowledge.player_mut(1).own_hand |= 1 << 1;

        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let gotten = pov.gotten_cards();
        assert!(gotten.contains(R2.as_variant_card_id()));
    }
}
