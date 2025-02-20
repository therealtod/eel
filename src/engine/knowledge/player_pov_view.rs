use crate::engine::knowledge::player_knowledge_state::PlayerKnowledgeState;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::engine::knowledge::team_knowledge::TeamKnowledge;
use crate::game::action::game_action::GameAction;
use crate::game::card::{CardDeckIndex, DeckCardsBitField, VariantCardId};
#[cfg(test)]
use crate::game::card::VariantCardsBitField;
use crate::game::state::PlayerIndex;
use crate::game::state::table_state::TableState;
use crate::game::static_game_data::StaticGameData;

/// Lightweight, read-only view that combines shared game state with player-specific knowledge.
///
/// Created on-the-fly when convention techs need to evaluate the game from a player's perspective.
/// Does **not** own any data — borrows everything from the single `TableState` and
/// per-player `PlayerKnowledgeState`.
pub struct PlayerPOVView<'a> {
    player_index: usize,
    knowledge: &'a PlayerKnowledgeState,
    team_knowledge: &'a TeamKnowledge,
    table_state: &'a TableState,
    static_data: &'a StaticGameData,
}

impl<'a> PlayerPOVView<'a> {
    pub fn new(
        player_index: usize,
        knowledge: &'a PlayerKnowledgeState,
        team_knowledge: &'a TeamKnowledge,
        table_state: &'a TableState,
        static_data: &'a StaticGameData,
    ) -> Self {
        PlayerPOVView {
            player_index,
            knowledge,
            team_knowledge,
            table_state,
            static_data,
        }
    }
}

impl PlayerPOV for PlayerPOVView<'_> {
    fn away_value(&self, card_id: VariantCardId) -> u8 {
        let stacks_size = self.static_data.variant.stacks_size as usize;
        let suit = card_id / stacks_size;
        let rank_idx = card_id % stacks_size;
        let stack_top = self.table_state.playing_stacks.stack_size(suit) as usize;
        rank_idx.saturating_sub(stack_top) as u8
    }

    fn card_identity(&self, card_deck_index: CardDeckIndex) -> Option<VariantCardId> {
        let empathy = self.knowledge.empathy[card_deck_index as usize];
        if empathy.count_ones() == 1 {
            Some(empathy.trailing_zeros() as VariantCardId)
        } else {
            None
        }
    }

    fn valid_actions(&self) -> Vec<GameAction> {
        todo!()
    }

    fn own_playable_cards(&self) -> DeckCardsBitField {
        let playable_cards = self.table_state.playable_cards(self.static_data);
        let own_hand = self.knowledge.own_hand;
        let mut result: DeckCardsBitField = 0;
        let mut hand_mask = own_hand;
        while hand_mask != 0 {
            let card_deck_index = hand_mask.trailing_zeros() as CardDeckIndex;
            let possible = self.knowledge.empathy[card_deck_index as usize];
            // A card is playable only if ALL its possible identities are playable
            if possible != 0 && (possible & playable_cards) == possible {
                result |= 1 << card_deck_index;
            }
            hand_mask &= !(1u64 << card_deck_index);
        }
        result
    }

    fn is_playable(&self, card_deck_index: CardDeckIndex) -> bool {
        let playable_cards = self.table_state.playable_cards(self.static_data);
        let possible = self.knowledge.empathy[card_deck_index as usize];
        possible != 0 && (possible & playable_cards) == possible
    }

    fn is_touched(&self, card_deck_index: CardDeckIndex) -> bool {
        self.table_state.clue_touched_cards & (1 << card_deck_index) != 0
    }

    fn is_identity_known_to_holder(&self, card_deck_index: CardDeckIndex) -> bool {
        let num_players = self.static_data.number_of_players as usize;
        (0..num_players).any(|p| {
            let pk = self.team_knowledge.player(p);
            pk.own_hand & (1 << card_deck_index) != 0
                && pk.visible_cards & (1 << card_deck_index) != 0
        })
    }

    fn is_critical(&self, card_deck_index: CardDeckIndex) -> bool {
        let Some(card_id) = self.card_identity(card_deck_index) else { return false };
        let total = self.static_data.variant.card_copies_count_by_id[card_id];
        let discarded = self.table_state.discard_pile.copies_of(card_id);
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::deck::unit_test_constant::novariant_constants::NoVarCards::*;
    use crate::game::deck::unit_test_constant::novariant_constants::*;
    use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
        initial_five_players_table_state, NOVAR_5_PLAYERS_STATIC_GAME_DATA,
    };

    fn knowledge_with_empathy(
        card_deck_index: CardDeckIndex,
        possible_identities: VariantCardsBitField,
    ) -> PlayerKnowledgeState {
        let mut k = PlayerKnowledgeState::new(0);
        k.empathy[card_deck_index as usize] = possible_identities;
        k.own_hand = 1 << card_deck_index;
        k
    }

    #[test]
    fn knows_that_a_fully_known_card_is_playable() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(0);
        table_state.update_with_draw_action(1);
        table_state.update_with_play_action_of_specific_card(0, R1.as_variant_card_id(), &static_data);
        table_state.update_with_play_action_of_specific_card(1, R2.as_variant_card_id(), &static_data);

        let knowledge = knowledge_with_empathy(42, R3_MASK);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let view = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(view.is_playable(42));
    }

    #[test]
    fn knows_that_a_fully_known_card_is_not_playable() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(0);
        table_state.update_with_draw_action(1);
        table_state.update_with_play_action_of_specific_card(0, R1.as_variant_card_id(), &static_data);
        table_state.update_with_play_action_of_specific_card(1, R2.as_variant_card_id(), &static_data);

        let knowledge = knowledge_with_empathy(42, B3_MASK);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let view = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(!view.is_playable(42));
    }

    #[test]
    fn knows_a_card_is_playable_because_all_possible_identities_are_playable() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(0);
        table_state.update_with_draw_action(1);
        table_state.update_with_draw_action(2);
        table_state.update_with_draw_action(3);
        table_state.update_with_play_action_of_specific_card(0, Y1.as_variant_card_id(), &static_data);
        table_state.update_with_play_action_of_specific_card(1, B1.as_variant_card_id(), &static_data);
        table_state.update_with_play_action_of_specific_card(2, B2.as_variant_card_id(), &static_data);
        table_state.update_with_play_action_of_specific_card(3, R2.as_variant_card_id(), &static_data);

        let knowledge = knowledge_with_empathy(42, R1_MASK | Y2_MASK | G1_MASK | B3_MASK | P1_MASK);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let view = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(view.is_playable(42));
    }

    #[test]
    fn a_card_is_not_playable_because_not_all_possible_identities_are_playable() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(0);
        table_state.update_with_draw_action(1);
        table_state.update_with_draw_action(2);
        table_state.update_with_draw_action(3);
        table_state.update_with_play_action_of_specific_card(0, Y1.as_variant_card_id(), &static_data);
        table_state.update_with_play_action_of_specific_card(1, B1.as_variant_card_id(), &static_data);
        table_state.update_with_play_action_of_specific_card(2, B2.as_variant_card_id(), &static_data);
        table_state.update_with_play_action_of_specific_card(3, R2.as_variant_card_id(), &static_data);

        let knowledge = knowledge_with_empathy(42, R1_MASK | Y2_MASK | Y3_MASK | G1_MASK | B3_MASK | P1_MASK);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let view = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(!view.is_playable(42));
    }
}
