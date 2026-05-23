use crate::engine::convention::convention_tech::ConventionTech;
use crate::engine::convention::hgroup::h_group_core::touched_cards_for_clue;
use crate::engine::game_state_snapshot::GameStateSnapshot;
use crate::engine::knowledge::knowledge_update::Hypothesis;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::clue::Clue;
use crate::game::clue_type::ClueType;

/// Stall: clue rank 5 to the first teammate who holds a 5 (any slot).
///
/// This is a fallback tech invoked when no higher-priority tech produces an action.
/// Unlike `FiveSave`, which requires the 5 to be on chop, `FiveStall` triggers on
/// any visible 5 in the target's hand.
pub struct FiveStall;

impl ConventionTech for FiveStall {
    fn name(&self) -> &'static str {
        "FiveStall"
    }

    fn interpretation_priority(&self) -> u8 {
        u8::MAX
    }

    fn game_actions(&self, pov: &dyn PlayerPOV) -> Vec<GameAction> {
        let active = pov.active_player_index();
        let table_state = pov.table_state();
        let static_data = pov.static_data();
        let num_players = static_data.number_of_players as usize;

        if table_state.clue_token_bank.whole_clue_tokens_count() == 0 {
            return vec![];
        }

        let rank5_clue = Clue {
            clue_type: ClueType::Rank,
            clue_value: 5,
        };
        let rank5_mask = static_data.variant.empathy_for_clue(&rank5_clue).as_bits();

        for target in (0..num_players).filter(|&p| p != active) {
            let has_five = table_state.hands[target].cards().iter().any(|&idx| {
                pov.card_identity(idx)
                    .is_some_and(|id| (1u64 << id) & rank5_mask != 0)
            });
            if has_five {
                let touched = touched_cards_for_clue(target, &rank5_clue, pov);
                if !touched.is_empty() {
                    return vec![GameAction::Clue {
                        player_index: target,
                        touched_card_deck_indexes: touched,
                        clue: rank5_clue,
                        turn: table_state.current_turn,
                    }];
                }
            }
        }

        vec![]
    }

    fn matches_action(
        &self,
        _action: &GameAction,
        _history: &[GameStateSnapshot],
        _observer_pov: &dyn PlayerPOV,
    ) -> bool {
        false
    }

    fn knowledge_updates(
        &self,
        _action: &GameAction,
        _history: &[GameStateSnapshot],
        _observer_pov: &dyn PlayerPOV,
    ) -> Hypothesis {
        Hypothesis::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::convention::convention_tech::ConventionTech;
    use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
    use crate::engine::knowledge::player_knowledge::knowledge_with_visible;
    use crate::engine::knowledge::team_knowledge::TeamKnowledge;
    use crate::game::clue_type::ClueType;
    use crate::game::deck::unit_test_constants::novariant_constants::{R5_MASK, Y5_MASK};
    use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
        NOVAR_5_PLAYERS_STATIC_GAME_DATA, initial_five_players_table_state,
    };
    use smallvec::smallvec;

    #[test]
    fn clues_rank5_to_teammate_holding_a_5() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.clue_token_bank.set_half_tokens(2); // 1 whole clue
        table_state.current_turn = 3;

        // Player 1 draws R5.
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R5_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = FiveStall.game_actions(&pov);

        assert_eq!(
            actions,
            vec![GameAction::Clue {
                player_index: 1,
                touched_card_deck_indexes: smallvec![10],
                clue: Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 5
                },
                turn: 3,
            }]
        );
    }

    #[test]
    fn rank5_clue_touches_all_fives_in_hand() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.clue_token_bank.set_half_tokens(2);
        table_state.current_turn = 4;

        // Player 1 has R5 (chop) and Y5 (newer).
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // R5, oldest
        table_state.update_with_draw_action(20); // Y5, newest
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R5_MASK), (20, Y5_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = FiveStall.game_actions(&pov);

        assert_eq!(actions.len(), 1);
        let touched = match &actions[0] {
            GameAction::Clue {
                touched_card_deck_indexes,
                ..
            } => touched_card_deck_indexes,
            _ => panic!("expected clue"),
        };
        assert!(touched.contains(&10));
        assert!(touched.contains(&20));
    }

    #[test]
    fn returns_empty_when_no_clue_tokens() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.clue_token_bank.set_half_tokens(0);
        table_state.current_turn = 5;

        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R5_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(FiveStall.game_actions(&pov).is_empty());
    }

    #[test]
    fn returns_empty_when_no_visible_5s() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.clue_token_bank.set_half_tokens(2);
        table_state.current_turn = 6;

        let knowledge = knowledge_with_visible(0, &[]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(FiveStall.game_actions(&pov).is_empty());
    }
}
