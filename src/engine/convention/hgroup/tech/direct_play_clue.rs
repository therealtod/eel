use crate::engine::convention::convention_tech::ClueTech;
use crate::engine::convention::hgroup::h_group_core::{
    clues_for_player_with_focus, get_clue_focus, has_pending_play_signal,
};
use crate::engine::convention::hgroup::h_group_tech::{HGroupClueTech, PlayClueTech, priority};
use crate::engine::game_state_snapshot::GameStateSnapshot;
use crate::engine::knowledge::knowledge_update::{Hypothesis, KnowledgeUpdate};
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::card::CardDeckIndex;
use crate::game::clue::Clue;
use crate::game::state::PlayerIndex;
use crate::impl_convention_tech_for_hgroup_clue_tech;

/// Give a clue to another player whose focus card is immediately playable on the stacks.
///
/// "Focus" follows H-Group rules: if the clue touches the receiver's chop, the chop is the
/// focus; otherwise the focus is the leftmost (newest, slot 1) newly-touched card.
///
/// A clue action is generated for every (target player, clue type, clue value) combination
/// whose focus card has a fully-known identity that is in `table_state.playable_cards()`.
///
/// # Limitation
/// Focus calculation uses the clue *giver's* POV to check `is_clued` on the receiver's cards.
/// The giver's knowledge does not track the receiver's convention signals, so a card in the
/// receiver's hand will be treated as unclued even if it was previously clued. This can produce
/// a wrong focus in re-clue scenarios; it is correct for freshly dealt hands.
pub struct DirectPlayClue;

impl DirectPlayClue {
    /// Core direct play detection: checks if the focus card is currently playable on the stacks.
    fn is_direct_play_situation(focus_idx: CardDeckIndex, pov: &dyn PlayerPOV) -> bool {
        pov.card_identity(focus_idx).is_some_and(|card_id| {
            (pov.table_state().playable_cards(pov.static_data()) >> card_id) & 1 != 0
                && !pov.is_gotten(card_id)
        })
    }
}

impl ClueTech for DirectPlayClue {
    fn clue_game_actions(&self, active_player_pov: &dyn PlayerPOV) -> Vec<GameAction> {
        let active_player_index = active_player_pov.active_player_index();
        let num_players = active_player_pov.static_data().number_of_players as usize;

        (0..num_players)
            .filter(|&x| x != active_player_index)
            .flat_map(|target| {
                clues_for_player_with_focus(target, active_player_pov)
                    .into_iter()
                    .filter_map(move |(action, focus_idx)| {
                        if Self::is_direct_play_situation(focus_idx, active_player_pov) {
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
        let Some(game_state_snapshot) = history.get(turn.saturating_sub(1)) else {
            return false;
        };
        let giver = game_state_snapshot.table_state.active_player_index;
        let giver_pov = game_state_snapshot.player_pov(giver, observer_pov.static_data());
        let Some(focus_idx) = get_clue_focus(player_index, touched, &giver_pov) else {
            return false;
        };
        // An observer matches this tech if, from their POV, the focus card could be
        // immediately playable. Observers who see the card (Alice, Bob) have a singleton
        // empathy; the receiver (Cathy) has a wide empathy narrowed only by the clue.
        //
        // Good-Touch tightening: a direct-play interpretation is incompatible with focus
        // identities already gotten elsewhere — the giver wouldn't pick this tech if the
        // focus could only be a duplicate. Strip those identities (other than the focus's
        // own, when the focus itself is touched, to allow re-clue scenarios).
        let static_data = observer_pov.static_data();
        let clue_mask = static_data.variant.empathy_for_clue(clue).as_bits();
        let playable = observer_pov.table_state().playable_cards(static_data);
        let gotten = observer_pov.gotten_cards().as_bits();
        let focus_own_gotten = observer_pov
            .card_identity(focus_idx)
            .filter(|_| observer_pov.is_touched(focus_idx))
            .map(|id| 1u64 << id)
            .unwrap_or(0);
        let external_gotten = gotten & !focus_own_gotten;
        (observer_pov.inferred_identities(focus_idx).as_bits()
            & clue_mask
            & playable
            & !external_gotten)
            != 0
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
        let Some(snap) = history.get(turn.saturating_sub(1)) else {
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
                    (1u64 << id) & clue_mask != 0 && away_value == 0
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

impl HGroupClueTech for DirectPlayClue {}
impl PlayClueTech for DirectPlayClue {}
impl_convention_tech_for_hgroup_clue_tech!(DirectPlayClue, priority::SIMPLE_PLAY_CLUE);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::convention::convention_tech::ConventionTech;
    use crate::engine::game_state_snapshot::GameStateSnapshot;
    use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
    use crate::engine::knowledge::player_knowledge::{PlayerKnowledge, knowledge_with_visible};
    use crate::engine::knowledge::team_knowledge::TeamKnowledge;
    use crate::game::action::game_action::GameAction;
    use crate::game::card::{CardDeckIndex, CardIdentityMask};
    use crate::game::clue::Clue;
    use crate::game::clue_type::ClueType;
    use crate::game::deck::unit_test_constants::novariant_constants::{R1_MASK, R2_MASK, Y1_MASK};
    use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
        NOVAR_5_PLAYERS_STATIC_GAME_DATA, initial_five_players_table_state,
    };
    use smallvec::smallvec;

    // ── game_actions ───────────────────────────────────────────────────────────

    #[test]
    fn returns_empty_when_no_playable_card_in_other_hands() {
        // Stacks are empty; R1 is playable. Player 1 only has R2 (not playable).
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // player 1 gets card 10 = R2
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R2_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(DirectPlayClue.game_actions(&pov).is_empty());
    }

    #[test]
    fn generates_clue_when_focus_is_the_only_card_and_it_is_playable() {
        // Player 1 has only R1 (card 10). With empty stacks R1 is playable.
        // Both the red clue and the "1" clue touch it — after dedup one action is returned.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // R1
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = DirectPlayClue.game_actions(&pov);

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
                    turn: 1
                },
                GameAction::Clue {
                    player_index: 1,
                    touched_card_deck_indexes: smallvec::smallvec![10],
                    clue: Clue {
                        clue_type: ClueType::Rank,
                        clue_value: 1,
                    },
                    turn: 1
                }
            ]
        );
    }

    #[test]
    fn does_not_generate_clue_when_focus_is_not_playable() {
        // Player 1: hand oldest→newest = [card 10 = R2 (chop), card 20 = R1].
        // Red clue touches both → chop R2 is focus → not playable → skip.
        // "1" clue touches only R1 (card 20) → focus = R1 → playable → generate.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // oldest = R2
        table_state.update_with_draw_action(20); // newest = R1
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R2_MASK), (20, R1_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = DirectPlayClue.game_actions(&pov);

        // Only the "1" clue action (touches [20] only)
        assert_eq!(
            actions,
            vec![GameAction::Clue {
                player_index: 1,
                touched_card_deck_indexes: smallvec::smallvec![20],
                clue: Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 1,
                },
                turn: 1
            }]
        );
    }

    #[test]
    fn generates_clues_for_multiple_target_players() {
        // Player 1 has R1 (card 10), player 2 has Y1 (card 20). Both playable.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // R1
        table_state.active_player_index = 2;
        table_state.update_with_draw_action(20); // Y1
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK), (20, Y1_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = DirectPlayClue.game_actions(&pov);

        assert!(actions.contains(&GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: smallvec::smallvec![10],
            clue: Clue {
                clue_type: ClueType::Color,
                clue_value: 0,
            },
            turn: 1
        }));
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
                    turn: 1
                },
                GameAction::Clue {
                    player_index: 1,
                    touched_card_deck_indexes: smallvec::smallvec![10],
                    clue: Clue {
                        clue_type: ClueType::Rank,
                        clue_value: 1,
                    },
                    turn: 1
                },
                GameAction::Clue {
                    player_index: 2,
                    touched_card_deck_indexes: smallvec::smallvec![20],
                    clue: Clue {
                        clue_type: ClueType::Color,
                        clue_value: 1,
                    },
                    turn: 1
                },
                GameAction::Clue {
                    player_index: 2,
                    touched_card_deck_indexes: smallvec::smallvec![20],
                    clue: Clue {
                        clue_type: ClueType::Rank,
                        clue_value: 1,
                    },
                    turn: 1
                },
            ]
        );
    }

    // ── matches_action ─────────────────────────────────────────────────────────

    #[test]
    fn matches_action_true_when_focus_is_playable() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // R1
        table_state.active_player_index = 0; // giver

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
        assert!(DirectPlayClue.matches_action(&clue, &[snapshot], &pov));
    }

    #[test]
    fn matches_action_false_when_focus_is_not_playable() {
        // R2 has away=1 with empty stacks, so it is not a direct play clue.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // R2
        table_state.active_player_index = 0; // giver

        let knowledge = knowledge_with_visible(0, &[(10, R2_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).inferred_identities[10] =
            Some(CardIdentityMask::from_bits(R2_MASK));
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
        assert!(!DirectPlayClue.matches_action(&clue, &[snapshot], &pov));
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
        assert!(!DirectPlayClue.matches_action(&play, &[], &pov));
    }

    #[test]
    fn matches_action_false_when_touched_is_empty_and_no_focus() {
        // An empty touched list → get_clue_focus returns None → false.
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
        assert!(!DirectPlayClue.matches_action(&clue, &[snapshot], &pov));
    }

    // ── knowledge_updates ──────────────────────────────────────────────────────

    #[test]
    fn knowledge_updates_narrows_focus_to_immediately_playable_ids() {
        // Stacks are empty. Player 1 (receiver) holds R1 (card 10). Giver (player 0) sees it.
        // A red clue touches only R1. The mask must contain R1 and exclude R2–R5 (not yet playable).
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // R1
        table_state.active_player_index = 0; // giver

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).inferred_identities[10] =
            Some(CardIdentityMask::from_bits(R1_MASK));
        team_knowledge.player_mut(0).visible_cards |= 1 << 10;

        let snapshot = GameStateSnapshot::new(table_state.clone(), team_knowledge.clone());
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let updates = DirectPlayClue.knowledge_updates(
            &GameAction::Clue {
                player_index: 1,
                touched_card_deck_indexes: smallvec::smallvec![10],
                clue: Clue {
                    clue_type: ClueType::Color,
                    clue_value: 0,
                },
                turn: 1,
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
            assert_ne!(mask & R1_MASK, 0, "R1 should be in the mask");
            assert_eq!(
                mask & R2_MASK,
                0,
                "R2 (not yet playable) should not be in the mask"
            );
        } else {
            panic!("expected NarrowPossibilities");
        }
    }

    #[test]
    fn knowledge_updates_returns_empty_when_no_touched_cards() {
        // An empty touched list → get_clue_focus returns None → Hypothesis::empty().
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(10);
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let snapshot = GameStateSnapshot::new(table_state.clone(), team_knowledge.clone());
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(
            DirectPlayClue
                .knowledge_updates(
                    &GameAction::Clue {
                        player_index: 0,
                        touched_card_deck_indexes: smallvec::smallvec![],
                        clue: Clue {
                            clue_type: ClueType::Color,
                            clue_value: 0
                        },
                        turn: 1,
                    },
                    &[snapshot],
                    &pov,
                )
                .is_empty()
        );
    }
}
