use crate::engine::convention::convention_tech::DiscardTech;
use crate::engine::convention::hgroup::h_group_core::get_chop_index;
use crate::engine::game_state_snapshot::GameStateSnapshot;
use crate::engine::knowledge::knowledge_update::Hypothesis;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::card::CardDeckIndex;
use crate::game::state::PlayerIndex;
use crate::impl_convention_tech_for_discard_tech;

/// Discard the chop card: the oldest unclued card in the active player's hand.
///
/// This is the standard H-Group fallback action when no clued cards are playable
/// and no clue needs to be given.  Cards appear in `Hand::cards()` oldest-first,
/// so the first unclued card in that slice is the chop.
///
/// If every card in hand has been clued (unusual but possible), no action is
/// returned — the convention does not apply.
pub struct DiscardChop;

impl DiscardTech for DiscardChop {
    fn discard_game_actions(&self, active_player_pov: &dyn PlayerPOV) -> Vec<GameAction> {
        let player_index = active_player_pov.active_player_index();
        match get_chop_index(player_index, active_player_pov) {
            Some(card_deck_index) => vec![GameAction::Discard {
                player_index,
                card_deck_index,
                turn: active_player_pov.table_state().current_turn,
            }],
            None => vec![],
        }
    }

    fn matches_discard(
        &self,
        player_index: PlayerIndex,
        card_deck_index: CardDeckIndex,
        _turn: usize,
        _history: &[GameStateSnapshot],
        observer_pov: &dyn PlayerPOV,
    ) -> bool {
        get_chop_index(player_index, observer_pov).as_ref() == Some(&card_deck_index)
    }

    fn discard_knowledge_updates(
        &self,
        _player_index: PlayerIndex,
        _card: CardDeckIndex,
        _turn: usize,
        _history: &[GameStateSnapshot],
        _observer_pov: &dyn PlayerPOV,
    ) -> Hypothesis {
        Hypothesis::empty()
    }
}

impl_convention_tech_for_discard_tech!(DiscardChop);

#[cfg(test)]
mod tests {
    mod integration {
        use crate::engine::convention::convention_tech::ConventionTech;
        use crate::engine::convention::hgroup::tech::discard_chop::DiscardChop;
        use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
        use crate::engine::knowledge::player_knowledge::knowledge_for_hand;
        use crate::engine::knowledge::team_knowledge::TeamKnowledge;
        use crate::game::action::game_action::GameAction;
        use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
            NOVAR_5_PLAYERS_STATIC_GAME_DATA, initial_five_players_table_state,
        };

        #[test]
        fn discards_oldest_card_when_all_unclued() {
            let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
            let mut table_state = initial_five_players_table_state();
            table_state.current_turn = 1; // Expected turn in action
            // Hand for player 0: cards drawn oldest→newest = [10, 20, 30, 40, 50]
            for &idx in &[10u8, 20, 30, 40, 50] {
                table_state.update_with_draw_action(idx);
            }
            let knowledge = knowledge_for_hand(&[10, 20, 30, 40, 50]);
            let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
            let pov = LightweightPlayerPOV::new(
                0,
                &knowledge,
                &team_knowledge,
                &table_state,
                &static_data,
            );

            let actions = DiscardChop.game_actions(&pov);

            assert_eq!(
                actions,
                vec![GameAction::Discard {
                    player_index: 0,
                    card_deck_index: 10,
                    turn: 1,
                }]
            );
        }

        #[test]
        fn skips_clued_cards_and_discards_oldest_unclued() {
            let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
            let mut table_state = initial_five_players_table_state();
            table_state.current_turn = 2; // Expected turn in action
            for &idx in &[10u8, 20, 30, 40, 50] {
                table_state.update_with_draw_action(idx);
            }
            // Mark card 10 (oldest) as touched by a clue
            table_state.clue_touched_cards |= 1 << 10;
            let knowledge = knowledge_for_hand(&[10, 20, 30, 40, 50]);

            let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
            let pov = LightweightPlayerPOV::new(
                0,
                &knowledge,
                &team_knowledge,
                &table_state,
                &static_data,
            );

            let actions = DiscardChop.game_actions(&pov);

            assert_eq!(
                actions,
                vec![GameAction::Discard {
                    player_index: 0,
                    card_deck_index: 20,
                    turn: 2
                }]
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
            let pov = LightweightPlayerPOV::new(
                0,
                &knowledge,
                &team_knowledge,
                &table_state,
                &static_data,
            );

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
            let pov = LightweightPlayerPOV::new(
                0,
                &knowledge,
                &team_knowledge,
                &table_state,
                &static_data,
            );

            let chop_action = GameAction::Discard {
                player_index: 0,
                card_deck_index: 10,
                turn: 5,
            };
            assert!(DiscardChop.matches_action(&chop_action, &[], &pov));
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
            let pov = LightweightPlayerPOV::new(
                0,
                &knowledge,
                &team_knowledge,
                &table_state,
                &static_data,
            );

            let non_chop = GameAction::Discard {
                player_index: 0,
                card_deck_index: 30,
                turn: 3,
            };
            assert!(!DiscardChop.matches_action(&non_chop, &[], &pov));
        }
    }
}
