use crate::engine::convention::convention_tech::ConventionTech;
use crate::engine::convention::hgroup::h_group_core::{get_chop_index, touched_cards_for_clue};
use crate::engine::knowledge::knowledge_update::KnowledgeUpdate;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::clue::Clue;
use crate::game::variant::RANK_CLUE_TYPE;

/// Clue rank 5 to a teammate whose chop card is a 5.
pub struct FiveSave;

const RANK_5_CLUE: Clue = Clue { clue_type: RANK_CLUE_TYPE as u8, clue_value: 5 };

impl ConventionTech for FiveSave {
    fn priority(&self) -> u8 {
        0
    }

    fn game_actions(&self, player_on_turn_pov: &dyn PlayerPOV) -> Vec<GameAction> {
        let active = player_on_turn_pov.player_on_turn_index();
        let static_data = player_on_turn_pov.static_data();
        let rank5_mask = static_data.variant.empathy_for_clue(&RANK_5_CLUE);
        let num_players = static_data.number_of_players as usize;

        (0..num_players)
            .filter(|&p| p != active)
            .filter_map(|target| {
                let chop = get_chop_index(target, player_on_turn_pov)?;
                let identity = player_on_turn_pov.card_identity(chop)?;
                if (1u64 << identity) & rank5_mask == 0 {
                    return None;
                }
                let touched = touched_cards_for_clue(target, &RANK_5_CLUE, player_on_turn_pov);
                Some(GameAction::Clue {
                    player_index: target,
                    touched_card_deck_indexes: touched,
                    clue: RANK_5_CLUE,
                })
            })
            .collect()
    }

    fn matches_action(&self, action: &GameAction, actor_pov: &dyn PlayerPOV) -> bool {
        if let GameAction::Clue { player_index, clue, .. } = action {
            *clue == RANK_5_CLUE
                && get_chop_index(*player_index, actor_pov)
                    .and_then(|chop| actor_pov.card_identity(chop))
                    .map(|id| {
                        let rank5_mask = actor_pov.static_data().variant.empathy_for_clue(&RANK_5_CLUE);
                        (1u64 << id) & rank5_mask != 0
                    })
                    .unwrap_or(false)
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
    use super::*;
    use crate::engine::convention::convention_tech::ConventionTech;
    use crate::engine::knowledge::player_knowledge_state::PlayerKnowledgeState;
    use crate::engine::knowledge::team_knowledge::TeamKnowledge;
    use crate::engine::knowledge::player_pov_view::PlayerPOVView;
    use crate::game::deck::unit_test_constant::novariant_constants::{R1_MASK, R5_MASK, Y5_MASK};
    use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
        initial_five_players_table_state, NOVAR_5_PLAYERS_STATIC_GAME_DATA,
    };

    fn knowledge_with_visible(player_index: usize, visible: &[(u8, u64)]) -> PlayerKnowledgeState {
        let mut k = PlayerKnowledgeState::new(player_index);
        for &(idx, mask) in visible {
            k.empathy[idx as usize] = mask;
            k.visible_cards |= 1 << idx;
        }
        k
    }

    #[test]
    fn generates_rank5_clue_when_chop_is_a_5() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // chop = R5
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R5_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = FiveSave.game_actions(&pov);

        assert_eq!(
            actions,
            vec![GameAction::Clue {
                player_index: 1,
                touched_card_deck_indexes: vec![10],
                clue: Clue { clue_type: 1, clue_value: 5 },
            }]
        );
    }

    #[test]
    fn returns_empty_when_chop_is_not_a_5() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // chop = R1
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(FiveSave.game_actions(&pov).is_empty());
    }

    #[test]
    fn touches_all_5s_in_hand_when_cluing() {
        // Player 1 has two 5s: R5 on chop (oldest) and Y5 (newest).
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // oldest = R5 (chop)
        table_state.update_with_draw_action(20); // newest = Y5
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R5_MASK), (20, Y5_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = FiveSave.game_actions(&pov);

        assert_eq!(actions.len(), 1);
        let touched = match &actions[0] {
            GameAction::Clue { touched_card_deck_indexes, .. } => touched_card_deck_indexes,
            _ => panic!("expected clue"),
        };
        assert!(touched.contains(&10));
        assert!(touched.contains(&20));
    }

    #[test]
    fn matches_action_true_when_chop_is_a_5_and_clue_is_rank5() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10);
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R5_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let action = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: vec![10],
            clue: Clue { clue_type: 1, clue_value: 5 },
        };
        assert!(FiveSave.matches_action(&action, &pov));
    }

    #[test]
    fn matches_action_false_when_chop_is_not_a_5() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10);
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let action = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: vec![10],
            clue: Clue { clue_type: 1, clue_value: 5 },
        };
        assert!(!FiveSave.matches_action(&action, &pov));
    }
}
