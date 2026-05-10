use crate::engine::convention::convention_tech::ClueTech;
use crate::engine::convention::hgroup::h_group_core::{
    clues_for_player_with_focus, get_clue_focus,
};
use crate::engine::convention::hgroup::h_group_tech::{HGroupClueTech, PlayClueTech, priority};
use crate::engine::game_state_snapshot::GameStateSnapshot;
use crate::engine::knowledge::knowledge_update::{Hypothesis, KnowledgeUpdate};
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::card::{CardDeckIndex, VariantCardId};
use crate::game::clue::Clue;
use crate::game::state::PlayerIndex;
use crate::impl_convention_tech_for_hgroup_clue_tech;

/// Give a clue whose focus card is not immediately playable but will become playable once all
/// connecting cards, which are already globally known to the team are played.
pub struct DelayedPlayClue;

impl DelayedPlayClue {
    /// Core delayed play detection: checks if the focus card is not immediately playable but will
    /// become playable once connecting cards are played (and all connecting cards are globally known).
    fn is_delayed_play_situation(card_id: VariantCardId, pov: &dyn PlayerPOV) -> bool {
        if let Some(away_value) = pov.away_value(card_id) {
            away_value > 0 && Self::connecting_cards_are_known(card_id, away_value, pov)
        } else {
            false
        }
    }

    fn connecting_cards_are_known(
        card_id: VariantCardId,
        away_value: u8,
        pov: &dyn PlayerPOV,
    ) -> bool {
        let active = pov.active_player_index();
        let num_players = pov.static_data().number_of_players as usize;

        (1..=away_value as usize).all(|offset| {
            let connecting_id = card_id - offset;
            (0..num_players).filter(|&p| p != active).any(|p| {
                pov.table_state().hands[p].cards().iter().any(|&idx| {
                    pov.card_identity(idx) == Some(connecting_id)
                        && pov.is_touched(idx)
                        && pov.is_identity_known_to_holder(idx)
                })
            })
        })
    }
}

impl ClueTech for DelayedPlayClue {
    fn clue_game_actions(&self, pov: &dyn PlayerPOV) -> Vec<GameAction> {
        let active = pov.active_player_index();
        let num_players = pov.static_data().number_of_players as usize;

        (0..num_players)
            .filter(|&p| p != active)
            .flat_map(|target| {
                clues_for_player_with_focus(target, pov)
                    .into_iter()
                    .filter_map(|(action, focus_idx)| {
                        let card_id = pov.card_identity(focus_idx)?;
                        if Self::is_delayed_play_situation(card_id, pov) {
                            Some(action)
                        } else {
                            None
                        }
                    })
            })
            .collect()
    }

    fn matches_clue(
        &self,
        player_index: PlayerIndex,
        touched: &[CardDeckIndex],
        clue: &Clue,
        turn: usize,
        history: &[GameStateSnapshot],
        observer_pov: &dyn PlayerPOV,
    ) -> bool {
        let Some(game_state_snapshot) = history.get(turn) else {
            return false;
        };
        let giver = game_state_snapshot.table_state.active_player_index;
        let giver_pov = game_state_snapshot.player_pov(giver, observer_pov.static_data());
        get_clue_focus(player_index, touched, &giver_pov)
            .and_then(|focus| giver_pov.card_identity(focus))
            .is_some_and(|card_id| Self::is_delayed_play_situation(card_id, &giver_pov))
    }

    fn clue_knowledge_updates(
        &self,
        player_index: PlayerIndex,
        touched: &[CardDeckIndex],
        clue: &Clue,
        turn: usize,
        history: &[GameStateSnapshot],
        observer_pov: &dyn PlayerPOV,
    ) -> Hypothesis {
        let Some(snap) = history.get(turn) else {
            return Hypothesis::empty();
        };
        let giver = snap.table_state.active_player_index;
        let giver_pov = snap.player_pov(giver, observer_pov.static_data());
        let focus = match get_clue_focus(player_index, touched, &giver_pov) {
            Some(f) => f,
            None => return Hypothesis::empty(),
        };
        let static_data = giver_pov.static_data();
        let total_ids =
            static_data.variant.number_of_suits as usize * static_data.variant.stacks_size as usize;
        let clue_mask = static_data.variant.empathy_for_clue(clue).as_bits();
        let mask: u64 = (0..total_ids)
            .filter(|&id| {
                if let Some(away_value) = giver_pov.away_value(id) {
                    (1u64 << id) & clue_mask != 0
                        && away_value > 0
                        && Self::connecting_cards_are_known(id, away_value, &giver_pov)
                } else {
                    false
                }
            })
            .fold(0u64, |acc, id| acc | (1 << id));
        if mask == 0 {
            return Hypothesis::empty();
        }
        Hypothesis::unconditional(vec![KnowledgeUpdate::NarrowPossibilities {
            card_deck_index: focus,
            mask,
        }])
    }
}

impl HGroupClueTech for DelayedPlayClue {}
impl PlayClueTech for DelayedPlayClue {}
impl_convention_tech_for_hgroup_clue_tech!(DelayedPlayClue, priority::SIMPLE_PLAY_CLUE);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::convention::convention_tech::ConventionTech;
    use crate::engine::game_state_snapshot::GameStateSnapshot;
    use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
    use crate::engine::knowledge::player_knowledge::{PlayerKnowledge, knowledge_with_visible};
    use crate::engine::knowledge::team_knowledge::TeamKnowledge;
    use crate::game::card::CardIdentityMask;
    use crate::game::clue::Clue;
    use crate::game::clue_type::ClueType;
    use crate::game::deck::unit_test_constants::novariant_constants::*;
    use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
        NOVAR_5_PLAYERS_STATIC_GAME_DATA, initial_five_players_table_state,
    };
    use smallvec::smallvec;

    // ── game_actions ───────────────────────────────────────────────────────────

    #[test]
    fn game_actions_returns_empty_when_no_connecting_card_is_visible() {
        // Player 1 has R3 (2-away). Connecting card R2 is not visible anywhere.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // R3
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R3_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(DelayedPlayClue.game_actions(&pov).is_empty());
    }

    #[test]
    fn game_actions_returns_empty_when_focus_is_immediately_playable() {
        // Player 1 has R1 (away=0). DirectPlayClue handles this; DelayedPlayClue should skip it.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // R1
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(DelayedPlayClue.game_actions(&pov).is_empty());
    }

    #[test]
    fn game_actions_generates_clue_when_connecting_card_is_visible_in_teammate_hand() {
        // R1 is played on the stack. Player 2 has R2 (card 20, touched+known). Player 1 has R3
        // (card 10). R3 is 1-away; connecting card R2 is touched and known to its holder, so
        // R3 is a valid delayed play clue target.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards::R1;
        table_state.update_with_play_action_of_specific_card(
            0,
            R1.as_variant_card_id(),
            &static_data,
        );
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // R3
        table_state.active_player_index = 2;
        table_state.update_with_draw_action(20); // R2
        table_state.clue_touched_cards |= 1 << 20; // R2 is touched by a clue
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R3_MASK), (20, R2_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        // Player 2 holds deck 20 and knows its identity (clued)
        team_knowledge.player_mut(2).own_hand |= 1 << 20;
        team_knowledge.player_mut(2).visible_cards |= 1 << 20;
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = DelayedPlayClue.game_actions(&pov);
        assert!(actions.iter().any(|a| matches!(
            a,
            GameAction::Clue {
                player_index: 1,
                ..
            }
        )));
        // Player 2's R2 is directly playable (R1 on stack), so it is NOT a delayed play clue target.
        assert!(actions.iter().all(|a| !matches!(
            a,
            GameAction::Clue {
                player_index: 2,
                ..
            }
        )));
    }

    #[test]
    fn game_actions_does_not_clue_own_player() {
        // R1 on the stack, player 0 has R3 (own hand), player 1 has R2 (touched + known to holder).
        // All conditions for a delayed play clue on R3 are satisfied, yet no clue targeting
        // player 0 is ever generated.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards::R1;
        table_state.update_with_play_action_of_specific_card(
            0,
            R1.as_variant_card_id(),
            &static_data,
        );
        table_state.active_player_index = 0;
        table_state.update_with_draw_action(10); // R3 in player 0's own hand
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(20); // R2 in player 1's hand
        table_state.clue_touched_cards |= 1 << 20; // R2 is touched by a clue
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R3_MASK), (20, R2_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(1).own_hand |= 1 << 20;
        team_knowledge.player_mut(1).visible_cards |= 1 << 20;
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = DelayedPlayClue.game_actions(&pov);
        assert!(actions.iter().all(|a| !matches!(
            a,
            GameAction::Clue {
                player_index: 0,
                ..
            }
        )));
    }

    // ── matches_action ─────────────────────────────────────────────────────────

    #[test]
    fn matches_action_true_when_focus_is_delayed_playable_with_connecting_card_visible() {
        // R1 is played on the stack. R3 (1-away) focus, R2 (connecting card) touched+known.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards::R1;
        table_state.update_with_play_action_of_specific_card(
            0,
            R1.as_variant_card_id(),
            &static_data,
        );
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // R3
        table_state.active_player_index = 2;
        table_state.update_with_draw_action(20); // R2
        table_state.clue_touched_cards |= 1 << 20; // R2 is touched by a clue
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R3_MASK), (20, R2_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).inferred_identities[10] =
            Some(CardIdentityMask::from_bits(R3_MASK));
        team_knowledge.player_mut(0).visible_cards |= 1 << 10;
        team_knowledge.player_mut(0).inferred_identities[20] =
            Some(CardIdentityMask::from_bits(R2_MASK));
        team_knowledge.player_mut(0).visible_cards |= 1 << 20;
        team_knowledge.player_mut(2).own_hand |= 1 << 20;
        team_knowledge.player_mut(2).visible_cards |= 1 << 20;

        let snapshot = GameStateSnapshot::new(table_state.clone(), team_knowledge.clone());
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let clue = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: smallvec::smallvec![10],
            clue: Clue {
                clue_type: ClueType::Color,
                clue_value: 0,
            }, // red clue
            turn: 0,
        };
        assert!(DelayedPlayClue.matches_action(&clue, &[snapshot], &pov));
    }

    #[test]
    fn matches_action_false_when_focus_is_immediately_playable() {
        // R1 has away=0, so it's not a delayed play clue.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // R1
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
                clue_type: ClueType::Rank,
                clue_value: 1,
            },
            turn: 0,
        };
        assert!(!DelayedPlayClue.matches_action(&clue, &[snapshot], &pov));
    }

    #[test]
    fn matches_action_false_when_connecting_card_not_visible() {
        // R3 is 2-away but no connecting card (R2) is touched or known anywhere.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // R3
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R3_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).inferred_identities[10] =
            Some(CardIdentityMask::from_bits(R3_MASK));
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
            turn: 0,
        };
        assert!(!DelayedPlayClue.matches_action(&clue, &[snapshot], &pov));
    }

    #[test]
    fn matches_action_false_for_non_clue_action() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let table_state = initial_five_players_table_state();
        let knowledge = PlayerKnowledge::new(0);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(!DelayedPlayClue.matches_action(
            &GameAction::Play {
                player_index: 0,
                card_deck_index: 5,
                turn: 1
            },
            &[],
            &pov
        ));
    }

    // ── knowledge_updates ──────────────────────────────────────────────────────

    #[test]
    fn knowledge_updates_narrows_focus_to_delayed_playable_ids() {
        // R1 is played on the stack. Player 0 (receiver) has R3 (card 10, touched).
        // Player 1 has R2 (card 20, visible) → R3 is a valid delayed play target (away=1, R2 visible).
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards::R1;
        table_state.update_with_play_action_of_specific_card(
            0,
            R1.as_variant_card_id(),
            &static_data,
        );
        table_state.active_player_index = 0;
        table_state.update_with_draw_action(10); // R3 in player 0's hand
        table_state.clue_touched_cards |= 1 << 10;
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(20); // R2 in player 1's hand
        table_state.clue_touched_cards |= 1 << 20; // R2 is touched by a clue
        table_state.active_player_index = 0; // Clue giver

        let knowledge = knowledge_with_visible(0, &[(10, R3_MASK), (20, R2_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).inferred_identities[10] =
            Some(CardIdentityMask::from_bits(R3_MASK));
        team_knowledge.player_mut(0).visible_cards |= 1u64 << 10;
        team_knowledge.player_mut(0).inferred_identities[20] =
            Some(CardIdentityMask::from_bits(R2_MASK));
        team_knowledge.player_mut(0).visible_cards |= 1u64 << 20;
        team_knowledge.player_mut(1).own_hand |= 1 << 20;
        team_knowledge.player_mut(1).visible_cards |= 1 << 20;

        let snapshot = GameStateSnapshot::new(table_state.clone(), team_knowledge.clone());
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let updates = DelayedPlayClue.knowledge_updates(
            &GameAction::Clue {
                player_index: 0,
                touched_card_deck_indexes: smallvec::smallvec![10],
                clue: Clue {
                    clue_type: ClueType::Color,
                    clue_value: 0,
                },
                turn: 0,
            },
            &[snapshot],
            &pov,
        );

        assert_eq!(updates.immediate.len(), 1);
        assert!(updates.trigger.is_none());
        if let KnowledgeUpdate::NarrowPossibilities {
            card_deck_index,
            mask,
        } = &updates.immediate[0]
        {
            assert_eq!(*card_deck_index, 10);
            // R3 (id=2) must be in the mask; R1 (id=0, away=0) and R2 (id=1, away=0 after R1 played)
            // must not be.
            assert_ne!(mask & R3_MASK, 0, "R3 should be in the mask");
            assert_eq!(mask & R1_MASK, 0, "R1 (played) should not be in the mask");
            assert_eq!(
                mask & R2_MASK,
                0,
                "R2 (immediately playable) should not be in the mask"
            );
        } else {
            panic!("expected NarrowPossibilities");
        }
    }

    #[test]
    fn knowledge_updates_returns_empty_when_no_touched_cards() {
        // get_clue_focus returns None for an empty touched list → no update.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(10);
        let knowledge = knowledge_with_visible(0, &[(10, R3_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let snapshot = GameStateSnapshot::new(table_state.clone(), team_knowledge.clone());
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(
            DelayedPlayClue
                .knowledge_updates(
                    &GameAction::Clue {
                        player_index: 0,
                        touched_card_deck_indexes: smallvec::smallvec![],
                        clue: Clue {
                            clue_type: ClueType::Color,
                            clue_value: 0
                        },
                        turn: 0
                    },
                    &[snapshot],
                    &pov
                )
                .is_empty()
        );
    }

    #[test]
    fn knowledge_updates_returns_empty_when_no_delayed_playable_ids_exist() {
        // All stacks complete → every card has away=None (already played) → mask is 0 → no update.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards::*;
        for &card_id in &[
            R1, R2, R3, R4, R5, Y1, Y2, Y3, Y4, Y5, G1, G2, G3, G4, G5, B1, B2, B3, B4, B5, P1, P2,
            P3, P4, P5,
        ] {
            table_state.update_with_play_action_of_specific_card(0, card_id as usize, &static_data);
        }
        table_state.active_player_index = 0;
        table_state.update_with_draw_action(10);
        table_state.clue_touched_cards |= 1 << 10;

        let knowledge = knowledge_with_visible(0, &[(10, R3_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let snapshot = GameStateSnapshot::new(table_state.clone(), team_knowledge.clone());
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(
            DelayedPlayClue
                .knowledge_updates(
                    &GameAction::Clue {
                        player_index: 1,
                        touched_card_deck_indexes: smallvec::smallvec![10],
                        clue: Clue {
                            clue_type: ClueType::Color,
                            clue_value: 0
                        },
                        turn: 0
                    },
                    &[snapshot],
                    &pov
                )
                .is_empty()
        );
    }
}
