use crate::engine::convention::hgroup::h_group_core::clues_for_player_with_focus;
use crate::engine::convention::hgroup::signal::Signal;
use crate::engine::knowledge::player_knowledge_state::PlayerKnowledgeState;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::engine::knowledge::team_knowledge::TeamKnowledge;
use crate::game::MAX_CLUE_TOKEN_COUNT;
use crate::game::action::game_action::GameAction;
use crate::game::card::{CardDeckIndex, DeckCardsBitField, VariantCardId};
use crate::game::state::PlayerIndex;
use crate::game::state::table_state::TableState;
use crate::game::static_game_data::StaticGameData;

/// Lightweight, read-only view that combines shared game state with player-specific knowledge.
///
/// Created on-the-fly when convention techs need to evaluate the game from a player's perspective.
/// Does **not** own any data — borrows everything from the single `TableState` and
/// per-player `PlayerKnowledgeState`.
pub struct LightweightPlayerPOV<'a> {
    player_index: usize,
    knowledge: &'a PlayerKnowledgeState,
    team_knowledge: &'a TeamKnowledge,
    table_state: &'a TableState,
    static_data: &'a StaticGameData,
}

impl<'a> LightweightPlayerPOV<'a> {
    pub fn new(
        player_index: usize,
        knowledge: &'a PlayerKnowledgeState,
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
        }
    }
}

impl PlayerPOV for LightweightPlayerPOV<'_> {
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
        // Use combined empathy: game-rule from Deck + convention-inferred from techs
        let combined = self.knowledge.combined_possible_identities(
            card_deck_index,
            self.table_state,
            &self.static_data.variant,
        );
        combined.known_card_id()
    }

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
            });
            if clue_tokens < MAX_CLUE_TOKEN_COUNT {
                actions.push(GameAction::Discard {
                    player_index,
                    card_deck_index,
                });
            }
            hand_mask &= !(1u64 << card_deck_index);
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

    fn own_playable_cards(&self) -> DeckCardsBitField {
        let playable_cards = self.table_state.playable_cards(self.static_data);
        let own_hand = self.knowledge.own_hand;
        let mut result: DeckCardsBitField = 0;
        let mut hand_mask = own_hand;
        while hand_mask != 0 {
            let card_deck_index = hand_mask.trailing_zeros() as CardDeckIndex;
            // Use combined empathy: game-rule from Deck + convention-inferred from techs
            let possible = self.knowledge.combined_possible_identities(
                card_deck_index,
                self.table_state,
                &self.static_data.variant,
            );
            // A card is playable if ALL its possible identities are playable (empathy-based),
            // OR if it has a Signal::Play (convention-inferred identity).
            let empathy_playable = (possible.as_bits() & playable_cards) == possible.as_bits();
            let signal_playable = self.knowledge.signals[card_deck_index as usize]
                .iter()
                .any(|s| matches!(s, Signal::Play { .. }));
            if empathy_playable || signal_playable {
                result |= 1 << card_deck_index;
            }
            hand_mask &= !(1u64 << card_deck_index);
        }
        result
    }

    fn is_playable(&self, card_deck_index: CardDeckIndex) -> bool {
        let playable_cards = self.table_state.playable_cards(self.static_data);
        let possible = self.knowledge.combined_possible_identities(
            card_deck_index,
            self.table_state,
            &self.static_data.variant,
        );
        let empathy_playable = (possible.as_bits() & playable_cards) == possible.as_bits();
        let signal_playable = self.knowledge.signals[card_deck_index as usize]
            .iter()
            .any(|s| matches!(s, Signal::Play { .. }));
        empathy_playable || signal_playable
    }

    fn is_touched(&self, card_deck_index: CardDeckIndex) -> bool {
        self.table_state.clue_touched_cards & (1 << card_deck_index) != 0
    }

    fn is_identity_known_to_holder(&self, card_deck_index: CardDeckIndex) -> bool {
        let num_players = self.static_data.number_of_players as usize;
        (0..num_players).any(|p| {
            let pk = self.team_knowledge.player(p);
            if pk.own_hand & (1 << card_deck_index) == 0 {
                return false;
            }
            // Known via direct identity reveal
            if pk.visible_cards & (1 << card_deck_index) != 0 {
                return true;
            }
            // Known via a play signal (e.g. finesse blind-play)
            pk.signals[card_deck_index as usize]
                .iter()
                .any(|s| matches!(s, Signal::Play { .. }))
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

    fn player_on_turn_index(&self) -> PlayerIndex {
        self.table_state.player_on_turn_index
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

    fn is_known_trash(&self, card_deck_index: CardDeckIndex) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::knowledge::player_knowledge_state::knowledge_with_empathy;
    use crate::game::action::game_action::GameAction;
    use crate::game::card::Empathy;
    use crate::game::clue_type::ClueType;
    use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards::*;
    use crate::game::deck::unit_test_constants::novariant_constants::*;
    use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
        NOVAR_5_PLAYERS_STATIC_GAME_DATA, initial_five_players_table_state,
    };

    #[test]
    fn ninth_clue_absent_from_valid_actions_after_full_bank_exhausted() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        // Player 0 has a card; player 1 has a known R1 so clues would otherwise be possible.
        table_state.update_with_draw_action(0);
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(1);
        table_state.player_on_turn_index = 0;

        // Spend all 8 tokens via update_with_clue_action (8 clues from a full bank).
        for _ in 0..8 {
            table_state.update_with_clue_action(
                smallvec::smallvec![1],
                crate::game::clue::Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 1,
                },
                1,
                &static_data,
            );
        }
        assert_eq!(table_state.clue_token_bank.whole_clue_tokens_count(), 0);

        let mut knowledge = PlayerKnowledgeState::new(0);
        knowledge.own_hand = 1 << 0u64;
        knowledge.inferred_identities[0] = Some(Empathy::from_bits(R1_MASK).unwrap());
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(1).inferred_identities[1] =
            Some(Empathy::from_bits(R2_MASK).unwrap());

        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);
        let actions = pov.valid_actions();

        assert!(
            actions
                .iter()
                .all(|a| !matches!(a, GameAction::Clue { .. })),
            "expected no Clue actions after 8 clues exhausted the bank, got: {actions:?}"
        );
    }

    #[test]
    fn valid_actions_with_zero_clue_tokens_emits_no_clue_actions() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        // Draw a card into player 0's hand and give player 1 a known card so clues would
        // otherwise be possible.
        table_state.update_with_draw_action(0);
        table_state.update_with_draw_action(1);
        // Zero out clue tokens.
        table_state.clue_token_bank = crate::game::clue_token_bank::ClueTokenBank::new(0);

        let mut knowledge = PlayerKnowledgeState::new(0);
        knowledge.own_hand = 1 << 0u64;
        knowledge.inferred_identities[0] = Some(Empathy::from_bits(R1_MASK).unwrap());
        // Give player 1 a known card so clues_for_player_with_focus would return something.
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(1).inferred_identities[1] =
            Some(Empathy::from_bits(R2_MASK).unwrap());

        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);
        let actions = pov.valid_actions();

        assert!(
            actions
                .iter()
                .all(|a| !matches!(a, GameAction::Clue { .. })),
            "expected no Clue actions when clue tokens == 0, got: {actions:?}"
        );
        assert!(
            !actions.is_empty(),
            "should still have Play/Discard actions"
        );
    }

    #[test]
    fn valid_actions_with_full_clue_tokens_emits_no_discard_actions() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(0);

        let mut knowledge = PlayerKnowledgeState::new(0);
        knowledge.own_hand = 1 << 0u64;
        knowledge.inferred_identities[0] = Some(Empathy::from_bits(R1_MASK).unwrap());
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);

        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);
        let actions = pov.valid_actions();

        assert!(
            actions
                .iter()
                .all(|a| !matches!(a, GameAction::Discard { .. })),
            "expected no Discard actions when clue tokens == MAX, got: {actions:?}"
        );
    }

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
}
