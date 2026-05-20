use crate::engine::convention::convention_tech::ConventionTech;
use crate::engine::convention::hgroup::h_group_core::{
    get_chop_index, still_needed_cards_mask, touched_cards_for_clue,
};
use crate::engine::game_state_snapshot::GameStateSnapshot;
use crate::engine::knowledge::knowledge_update::Hypothesis;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::MAX_CLUE_TOKEN_COUNT;
use crate::game::action::game_action::GameAction;
use crate::game::clue::Clue;
use crate::game::clue_type::ClueType;

/// Last-resort stall: invoked when no other tech produces an action.
///
/// Attempts the following in priority order and returns the first that succeeds:
/// 1. Clue rank 5 to the first teammate who holds a 5 (any slot).
/// 2. Clue the rank matching the chop card to the first teammate whose chop is useful
///    (still needed by the stacks).
/// 3. Discard slot 1 (when the clue token bank is not full).
/// 4. Play slot 1 (the newest card in the active player's hand).
///
/// This tech is **not** registered in `HGroupConventionSet::default()`; it is invoked
/// directly from the fallback path in `candidate_actions_with_provenance`.
pub struct LowLevelStall;

impl LowLevelStall {
    fn rank_clue_value(card_id: usize, stacks_size: usize) -> u8 {
        (card_id % stacks_size) as u8 + 1
    }
}

impl ConventionTech for LowLevelStall {
    fn name(&self) -> &'static str {
        "LowLevelStall"
    }

    fn interpretation_priority(&self) -> u8 {
        u8::MAX
    }

    fn game_actions(&self, pov: &dyn PlayerPOV) -> Vec<GameAction> {
        let active = pov.active_player_index();
        let table_state = pov.table_state();
        let static_data = pov.static_data();
        let num_players = static_data.number_of_players as usize;
        let stacks_size = static_data.variant.stacks_size as usize;
        let has_clues = table_state.clue_token_bank.whole_clue_tokens_count() > 0;

        if has_clues {
            let rank5_clue = Clue {
                clue_type: ClueType::Rank,
                clue_value: 5,
            };
            let rank5_mask = static_data.variant.empathy_for_clue(&rank5_clue).as_bits();

            // Priority 1: clue rank 5 to the first teammate who holds a 5.
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

            // Priority 2: clue the rank of the first useful chop card seen on a teammate.
            let still_needed = still_needed_cards_mask(table_state, static_data);
            for target in (0..num_players).filter(|&p| p != active) {
                let Some(chop_idx) = get_chop_index(target, pov) else {
                    continue;
                };
                let Some(card_id) = pov.card_identity(chop_idx) else {
                    continue;
                };
                if (1u64 << card_id) & still_needed == 0 {
                    continue;
                }
                let rank = Self::rank_clue_value(card_id, stacks_size);
                let clue = Clue {
                    clue_type: ClueType::Rank,
                    clue_value: rank,
                };
                let touched = touched_cards_for_clue(target, &clue, pov);
                if !touched.is_empty() {
                    return vec![GameAction::Clue {
                        player_index: target,
                        touched_card_deck_indexes: touched,
                        clue,
                        turn: table_state.current_turn,
                    }];
                }
            }
        }

        // Priority 3: discard slot 1 when the clue token bank is not full.
        if table_state.clue_token_bank.whole_clue_tokens_count() < MAX_CLUE_TOKEN_COUNT {
            if let Some(&slot1) = table_state.hands[active].cards().first() {
                return vec![GameAction::Discard {
                    player_index: active,
                    card_deck_index: slot1,
                    turn: table_state.current_turn,
                }];
            }
        }

        // Priority 4: play slot 1 (newest card in own hand).
        if let Some(&slot1) = table_state.hands[active].cards().first() {
            return vec![GameAction::Play {
                player_index: active,
                card_deck_index: slot1,
                turn: table_state.current_turn,
            }];
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
    use crate::game::deck::unit_test_constants::novariant_constants::{R5_MASK, Y1_MASK, Y5_MASK};
    use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
        NOVAR_5_PLAYERS_STATIC_GAME_DATA, initial_five_players_table_state,
    };
    use smallvec::smallvec;

    // ── Priority 1: rank-5 clue ───────────────────────────────────────────────

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

        let actions = LowLevelStall.game_actions(&pov);

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

        let actions = LowLevelStall.game_actions(&pov);

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

    // ── Priority 2: useful-chop rank clue ────────────────────────────────────

    #[test]
    fn clues_rank_of_useful_chop_when_no_5_visible() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.clue_token_bank.set_half_tokens(2);
        table_state.current_turn = 5;

        // Player 1 has Y1 on chop. Y1 is useful (stacks empty).
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, Y1_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = LowLevelStall.game_actions(&pov);

        assert_eq!(actions.len(), 1);
        assert!(matches!(
            &actions[0],
            GameAction::Clue { clue, .. } if clue.clue_type == ClueType::Rank && clue.clue_value == 1
        ));
    }

    // ── Priority 3: discard slot 1 ───────────────────────────────────────────

    #[test]
    fn discards_slot1_when_bank_not_full_and_no_clue_target() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        // 3 whole tokens — not full, not zero; no clue targets visible.
        table_state.clue_token_bank.set_half_tokens(6);
        table_state.current_turn = 6;

        // Player 0 draws card 10 (slot 1).
        table_state.update_with_draw_action(10);

        let knowledge = knowledge_with_visible(0, &[]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = LowLevelStall.game_actions(&pov);

        assert_eq!(
            actions,
            vec![GameAction::Discard {
                player_index: 0,
                card_deck_index: 10,
                turn: 6,
            }]
        );
    }

    #[test]
    fn discards_newest_card_as_slot1() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.clue_token_bank.set_half_tokens(6);
        table_state.current_turn = 7;

        // Player 0 draws 10 (older) then 20 (newer = slot 1).
        table_state.update_with_draw_action(10);
        table_state.update_with_draw_action(20);

        let knowledge = knowledge_with_visible(0, &[]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = LowLevelStall.game_actions(&pov);

        assert_eq!(
            actions,
            vec![GameAction::Discard {
                player_index: 0,
                card_deck_index: 20,
                turn: 7,
            }]
        );
    }

    // ── Priority 4: play slot 1 (bank full, cannot discard) ──────────────────

    #[test]
    fn plays_slot1_when_bank_is_full() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        // 8 whole tokens = full bank; discard is illegal.
        table_state.clue_token_bank.set_half_tokens(16);
        table_state.current_turn = 8;

        // Player 0 draws card 10 (slot 1). No teammate cards visible → no clue target.
        table_state.update_with_draw_action(10);

        let knowledge = knowledge_with_visible(0, &[]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = LowLevelStall.game_actions(&pov);

        assert_eq!(
            actions,
            vec![GameAction::Play {
                player_index: 0,
                card_deck_index: 10,
                turn: 8,
            }]
        );
    }
}
