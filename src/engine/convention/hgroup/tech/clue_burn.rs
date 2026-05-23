use crate::engine::convention::convention_tech::ClueTech;
use crate::engine::convention::hgroup::game_action_filter::GameActionFilter;
use crate::engine::convention::hgroup::h_group_core::touched_cards_for_clue;
use crate::engine::convention::hgroup::h_group_tech::{HGroupClueTech, priority};
use crate::engine::game_state_snapshot::GameStateSnapshot;
use crate::engine::knowledge::knowledge_update::Hypothesis;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::card::CardDeckIndex;
use crate::game::clue::Clue;
use crate::game::state::PlayerIndex;
use crate::game::{MAX_CLUE_VALUES_PER_TYPE};
use crate::impl_convention_tech_for_hgroup_clue_tech;

/// `ClueBurn`: a clue where every touched card is already touched (clue-touched or signal-touched).
///
/// This is a "burn" — it conveys no new information and serves only to consume a clue token.
/// It is treated as a very low-priority fallback: higher-priority techs (save, play clue, etc.)
/// are checked first during interpretation.
pub struct ClueBurn;

impl ClueTech for ClueBurn {
    fn clue_game_actions(&self, pov: &dyn PlayerPOV) -> Vec<GameAction> {
        let active = pov.active_player_index();
        let num_players = pov.static_data().number_of_players as usize;
        let static_data = pov.static_data();

        (0..num_players)
            .filter(|&p| p != active)
            .flat_map(|target| {
                let mut result = Vec::new();
                for clue_type in &static_data.variant.clue_types {
                    for clue_value in 0..MAX_CLUE_VALUES_PER_TYPE {
                        #[allow(clippy::cast_possible_truncation)]
                        let clue = Clue {
                            clue_type: *clue_type,
                            clue_value: clue_value as u8,
                        };
                        let touched = touched_cards_for_clue(target, &clue, pov);
                        if touched.is_empty() {
                            continue;
                        }
                        if touched.iter().all(|&idx| pov.is_touched(idx)) {
                            result.push(GameAction::Clue {
                                player_index: target,
                                touched_card_deck_indexes: touched,
                                clue,
                                turn: pov.table_state().current_turn,
                            });
                        }
                    }
                }
                result
            })
            .collect()
    }

    fn matches_clue(
        &self,
        _player_index: PlayerIndex,
        touched: &[CardDeckIndex],
        _clue: &Clue,
        turn: usize,
        history: &[GameStateSnapshot],
        observer_pov: &dyn PlayerPOV,
    ) -> bool {
        if touched.is_empty() {
            return false;
        }
        let Some(snap) = history.get(turn.saturating_sub(1)) else {
            return false;
        };
        let giver = snap.table_state.active_player_index;
        let giver_pov = snap.player_pov(giver, observer_pov.static_data());
        touched.iter().all(|&idx| giver_pov.is_touched(idx))
    }

    fn clue_knowledge_updates(
        &self,
        _player_index: PlayerIndex,
        _touched: &[CardDeckIndex],
        _clue: &Clue,
        _turn: usize,
        _history: &[GameStateSnapshot],
        _observer_pov: &dyn PlayerPOV,
    ) -> Hypothesis {
        Hypothesis::empty()
    }
}

impl HGroupClueTech for ClueBurn {
    fn clue_action_filters(&self) -> Vec<GameActionFilter> {
        vec![]
    }
}

impl_convention_tech_for_hgroup_clue_tech!(ClueBurn, priority::BURN_CLUE);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::convention::convention_tech::ConventionTech;
    use crate::engine::convention::hgroup::signal::Signal;
    use crate::engine::game_state_snapshot::GameStateSnapshot;
    use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
    use crate::engine::knowledge::player_knowledge::{PlayerKnowledge, knowledge_with_visible};
    use crate::engine::knowledge::team_knowledge::TeamKnowledge;
    use crate::game::card::CardIdentityMask;
    use crate::game::clue::Clue;
    use crate::game::clue_type::ClueType;
    use crate::game::deck::unit_test_constants::novariant_constants::{R1_MASK, R2_MASK};
    use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
        NOVAR_5_PLAYERS_STATIC_GAME_DATA, initial_five_players_table_state,
    };

    // ── game_actions ───────────────────────────────────────────────────────────

    #[test]
    fn returns_empty_when_no_card_is_touched() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(ClueBurn.game_actions(&pov).is_empty());
    }

    #[test]
    fn generates_clue_when_touched_card_is_already_clue_touched() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        table_state.active_player_index = 0;
        table_state.clue_touched_cards |= 1 << 10;

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = ClueBurn.game_actions(&pov);

        assert_eq!(
            actions,
            vec![
                GameAction::Clue {
                    player_index: 1,
                    touched_card_deck_indexes: smallvec::smallvec![10],
                    clue: Clue {
                        clue_type: ClueType::Color,
                        clue_value: 0,
                    },
                    turn: 1,
                },
                GameAction::Clue {
                    player_index: 1,
                    touched_card_deck_indexes: smallvec::smallvec![10],
                    clue: Clue {
                        clue_type: ClueType::Rank,
                        clue_value: 1,
                    },
                    turn: 1,
                },
            ]
        );
    }

    #[test]
    fn generates_clue_when_touched_card_is_already_signal_touched() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let tk = team_knowledge.player_mut(1);
        tk.own_hand |= 1 << 10;
        tk.add_signal(
            10,
            Signal::Play {
                card_deck_index: 10,
                committed_identity: 0,
            },
        );
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = ClueBurn.game_actions(&pov);

        assert_eq!(actions.len(), 2);
        assert!(actions.iter().all(|a| matches!(a, GameAction::Clue { player_index: 1, .. })));
    }

    #[test]
    fn skips_clue_that_touches_both_touched_and_untouched_cards() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        table_state.update_with_draw_action(20);
        table_state.active_player_index = 0;
        table_state.clue_touched_cards |= 1 << 10;

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK), (20, R2_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = ClueBurn.game_actions(&pov);

        // Red clue touches both R1 (touched) and R2 (untouched) — should be filtered out.
        assert!(!actions.iter().any(|a| matches!(a,
            GameAction::Clue { clue, .. } if clue.clue_type == ClueType::Color
        )));
        // Rank-2 clue touches only R2 (untouched) — should be filtered out.
        assert!(!actions.iter().any(|a| matches!(a,
            GameAction::Clue { clue, .. } if clue.clue_type == ClueType::Rank && clue.clue_value == 2
        )));
        // Rank-1 clue touches only R1 (touched) — should be generated.
        assert!(actions.iter().any(|a| matches!(a,
            GameAction::Clue { clue, .. } if clue.clue_type == ClueType::Rank && clue.clue_value == 1
        )));
    }

    // ── matches_action ─────────────────────────────────────────────────────────

    #[test]
    fn matches_action_true_when_all_touched_cards_were_already_touched() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        table_state.active_player_index = 0;
        table_state.clue_touched_cards |= 1 << 10;

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let snapshot = GameStateSnapshot::new(table_state.clone(), team_knowledge.clone());
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let clue = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: smallvec::smallvec![10],
            clue: Clue {
                clue_type: ClueType::Color,
                clue_value: 0,
            },
            turn: 1,
        };
        assert!(ClueBurn.matches_action(&clue, &[snapshot], &pov));
    }

    #[test]
    fn matches_action_false_when_touched_card_was_not_yet_touched() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).inferred_identities[10] =
            Some(CardIdentityMask::from_bits(R1_MASK));
        team_knowledge.player_mut(0).visible_cards |= 1 << 10;
        let snapshot = GameStateSnapshot::new(table_state.clone(), team_knowledge.clone());
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let clue = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: smallvec::smallvec![10],
            clue: Clue {
                clue_type: ClueType::Color,
                clue_value: 0,
            },
            turn: 1,
        };
        assert!(!ClueBurn.matches_action(&clue, &[snapshot], &pov));
    }

    #[test]
    fn matches_action_false_for_non_clue_action() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let table_state = initial_five_players_table_state();
        let knowledge = PlayerKnowledge::new(0);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let play = GameAction::Play {
            player_index: 0,
            card_deck_index: 5,
            turn: 1,
        };
        assert!(!ClueBurn.matches_action(&play, &[], &pov));
    }

    #[test]
    fn matches_action_false_when_touched_is_empty() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let table_state = initial_five_players_table_state();
        let knowledge = PlayerKnowledge::new(0);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let snapshot = GameStateSnapshot::new(table_state.clone(), team_knowledge.clone());
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let clue = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: smallvec::smallvec![],
            clue: Clue {
                clue_type: ClueType::Color,
                clue_value: 0,
            },
            turn: 1,
        };
        assert!(!ClueBurn.matches_action(&clue, &[snapshot], &pov));
    }

    #[test]
    fn matches_action_false_when_history_is_empty() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let table_state = initial_five_players_table_state();
        let knowledge = PlayerKnowledge::new(0);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let clue = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: smallvec::smallvec![10],
            clue: Clue {
                clue_type: ClueType::Color,
                clue_value: 0,
            },
            turn: 1,
        };
        assert!(!ClueBurn.matches_action(&clue, &[], &pov));
    }

    // ── knowledge_updates ──────────────────────────────────────────────────────

    #[test]
    fn knowledge_updates_always_empty_for_burn_clue() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let table_state = initial_five_players_table_state();
        let knowledge = PlayerKnowledge::new(0);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(
            ClueBurn
                .knowledge_updates(
                    &GameAction::Clue {
                        player_index: 0,
                        touched_card_deck_indexes: smallvec::smallvec![5],
                        clue: Clue {
                            clue_type: ClueType::Color,
                            clue_value: 0
                        },
                        turn: 1,
                    },
                    &[],
                    &pov,
                )
                .is_empty()
        );
    }
}
