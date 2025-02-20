use crate::engine::convention::convention_tech::ConventionTech;
use crate::engine::convention::hgroup::h_group_core::get_chop_index;
use crate::engine::knowledge::knowledge_update::KnowledgeUpdate;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;

/// Discard the chop card: the oldest unclued card in the active player's hand.
///
/// This is the standard H-Group fallback action when no clued cards are playable
/// and no clue needs to be given.  Cards appear in `Hand::cards()` oldest-first,
/// so the first unclued card in that slice is the chop.
///
/// If every card in hand has been clued (unusual but possible), no action is
/// returned — the convention does not apply.
pub struct DiscardChop;

impl ConventionTech for DiscardChop {
    fn priority(&self) -> u8 { 0 }

    fn game_actions(&self, player_on_turn_pov: &dyn PlayerPOV) -> Vec<GameAction> {
        let player_index = player_on_turn_pov.player_on_turn_index();
        match get_chop_index(player_index, player_on_turn_pov) {
            Some(card_deck_index) => vec![GameAction::Discard { player_index, card_deck_index }],
            None => vec![],
        }
    }

    fn matches_action(&self, action: &GameAction, actor_pov: &dyn PlayerPOV) -> bool {
        if let GameAction::Discard { card_deck_index, .. } = action {
            let player_index = actor_pov.player_on_turn_index();
            get_chop_index(player_index, actor_pov).as_ref() == Some(card_deck_index)
        } else {
            false
        }
    }

    fn knowledge_updates(&self, _player_pov: &dyn PlayerPOV) -> Vec<KnowledgeUpdate> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    mod integration {
        use crate::engine::convention::convention_tech::ConventionTech;
        use crate::engine::convention::hgroup::tech::discard_chop::DiscardChop;
        use crate::engine::knowledge::player_knowledge_state::PlayerKnowledgeState;
        use crate::engine::knowledge::team_knowledge::TeamKnowledge;
        use crate::engine::knowledge::player_pov_view::PlayerPOVView;
        use crate::game::action::game_action::GameAction;
        use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
            initial_five_players_table_state, NOVAR_5_PLAYERS_STATIC_GAME_DATA,
        };

        fn knowledge_for_hand(cards: &[u8]) -> PlayerKnowledgeState {
            let mut k = PlayerKnowledgeState::new(0);
            for &idx in cards {
                k.own_hand |= 1 << idx;
            }
            k
        }

        #[test]
        fn discards_oldest_card_when_all_unclued() {
            let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
            let mut table_state = initial_five_players_table_state();
            // Hand for player 0: cards drawn oldest→newest = [10, 20, 30, 40, 50]
            for &idx in &[10u8, 20, 30, 40, 50] {
                table_state.update_with_draw_action(idx);
            }
            let knowledge = knowledge_for_hand(&[10, 20, 30, 40, 50]);
            let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

            let actions = DiscardChop.game_actions(&pov);

            assert_eq!(
                actions,
                vec![GameAction::Discard { player_index: 0, card_deck_index: 10 }]
            );
        }

        #[test]
        fn skips_clued_cards_and_discards_oldest_unclued() {
            let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
            let mut table_state = initial_five_players_table_state();
            for &idx in &[10u8, 20, 30, 40, 50] {
                table_state.update_with_draw_action(idx);
            }
            // Mark card 10 (oldest) as touched by a clue
            table_state.clue_touched_cards |= 1 << 10;
            let knowledge = knowledge_for_hand(&[10, 20, 30, 40, 50]);

            let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

            let actions = DiscardChop.game_actions(&pov);

            assert_eq!(
                actions,
                vec![GameAction::Discard { player_index: 0, card_deck_index: 20 }]
            );
        }

        #[test]
        fn returns_no_action_when_all_cards_are_clued() {
            let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
            let mut table_state = initial_five_players_table_state();
            for &idx in &[10u8, 20] {
                table_state.update_with_draw_action(idx);
            }
            table_state.clue_touched_cards |= (1 << 10) | (1 << 20);
            let knowledge = knowledge_for_hand(&[10, 20]);

            let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

            assert!(DiscardChop.game_actions(&pov).is_empty());
        }

        #[test]
        fn matches_action_returns_true_for_chop_discard() {
            let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
            let mut table_state = initial_five_players_table_state();
            for &idx in &[10u8, 20, 30] {
                table_state.update_with_draw_action(idx);
            }
            let knowledge = knowledge_for_hand(&[10, 20, 30]);
            let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

            let chop_action = GameAction::Discard { player_index: 0, card_deck_index: 10 };
            assert!(DiscardChop.matches_action(&chop_action, &pov));
        }

        #[test]
        fn matches_action_returns_false_for_non_chop_discard() {
            let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
            let mut table_state = initial_five_players_table_state();
            for &idx in &[10u8, 20, 30] {
                table_state.update_with_draw_action(idx);
            }
            let knowledge = knowledge_for_hand(&[10, 20, 30]);
            let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

            let non_chop = GameAction::Discard { player_index: 0, card_deck_index: 30 };
            assert!(!DiscardChop.matches_action(&non_chop, &pov));
        }
    }
}
