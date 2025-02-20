use crate::engine::convention::convention_tech::ConventionTech;
use crate::engine::convention::hgroup::h_group_core::{clues_for_player_with_focus, get_clue_focus_index};
use crate::engine::knowledge::knowledge_update::KnowledgeUpdate;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::card::CardDeckIndex;
use crate::game::clue::Clue;

/// Give a clue to another player whose focus card is immediately playable on the stacks.
///
/// "Focus" follows H-Group rules: if the clue touches the receiver's chop, the chop is the
/// focus; otherwise the focus is the leftmost (newest, slot 1) newly-touched card.
///
/// A clue action is generated for every (target player, clue type, clue value) combination
/// whose focus card has a fully-known identity that is in `table_state.playable_cards()`.
/// Duplicate actions (same touched set, different clue type) are deduplicated before returning.
///
/// # Limitation
/// Focus calculation uses the clue *giver's* POV to check `is_clued` on the receiver's cards.
/// The giver's knowledge does not track the receiver's convention signals, so a card in the
/// receiver's hand will be treated as unclued even if it was previously clued. This can produce
/// a wrong focus in re-clue scenarios; it is correct for freshly dealt hands.
pub struct DirectPlayClue;

impl ConventionTech for DirectPlayClue {
    fn priority(&self) -> u8 {
        0
    }

    fn game_actions(&self, player_on_turn_pov: &dyn PlayerPOV) -> Vec<GameAction> {
        let active_player_index = player_on_turn_pov.player_on_turn_index();
        let table_state = player_on_turn_pov.table_state();
        let static_data = player_on_turn_pov.static_data();
        let num_players = static_data.number_of_players as usize;
        let playable_variant_cards = table_state.playable_cards(static_data);

        (0..num_players)
            .filter(|&x| x != active_player_index)
            .flat_map(|target| {
                clues_for_player_with_focus(target, player_on_turn_pov)
                    .into_iter()
                    .filter_map(|(action, focus_idx)| {
                        player_on_turn_pov
                            .card_identity(focus_idx)
                            .filter(|&id| (1u64 << id) & playable_variant_cards != 0)
                            .map(|_| action)
                    })
            })
            .collect()
    }

    /// Returns `true` if the given clue action's focus card is immediately playable from the
    /// actor's POV. The actor must be able to see the receiver's cards for this to be accurate.
    fn matches_action(&self, action: &GameAction, actor_pov: &dyn PlayerPOV) -> bool {
        if let GameAction::Clue {
            player_index,
            touched_card_deck_indexes,
            ..
        } = action
        {
            get_clue_focus_index(*player_index, touched_card_deck_indexes, actor_pov)
                .map(|focus_idx| actor_pov.is_playable(focus_idx))
                .unwrap_or(false)
        } else {
            false
        }
    }

    fn knowledge_updates(&self, player_pov: &dyn PlayerPOV) -> Vec<KnowledgeUpdate> {
        let receiver = player_pov.player_on_turn_index();
        let touched: Vec<CardDeckIndex> = player_pov.table_state().hands[receiver]
            .cards()
            .iter()
            .copied()
            .filter(|&idx| player_pov.is_touched(idx))
            .collect();
        let focus = match get_clue_focus_index(receiver, &touched, player_pov) {
            Some(f) => f,
            None => return vec![],
        };
        let mask = player_pov.table_state().playable_cards(player_pov.static_data());
        if mask == 0 { return vec![]; }
        vec![KnowledgeUpdate::NarrowPossibilities { card_deck_index: focus, mask }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::convention::convention_tech::ConventionTech;
    use crate::engine::knowledge::player_knowledge_state::PlayerKnowledgeState;
    use crate::engine::knowledge::team_knowledge::TeamKnowledge;
    use crate::engine::knowledge::player_pov_view::PlayerPOVView;
    use crate::game::action::game_action::GameAction;
    use crate::game::deck::unit_test_constant::novariant_constants::{R1_MASK, R2_MASK, Y1_MASK};
    use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
        initial_five_players_table_state, NOVAR_5_PLAYERS_STATIC_GAME_DATA,
    };

    /// Build a `PlayerKnowledgeState` for `player_index` that has the given deck cards
    /// as fully-known visible identities.
    fn knowledge_with_visible(
        player_index: usize,
        visible: &[(u8, u64)], // (card_deck_index, empathy_mask)
    ) -> PlayerKnowledgeState {
        let mut k = PlayerKnowledgeState::new(player_index);
        for &(idx, mask) in visible {
            k.empathy[idx as usize] = mask;
            k.visible_cards |= 1 << idx;
        }
        k
    }

    // ── game_actions ───────────────────────────────────────────────────────────

    #[test]
    fn returns_empty_when_no_playable_card_in_other_hands() {
        // Stacks are empty; R1 is playable. Player 1 only has R2 (not playable).
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // player 1 gets card 10 = R2
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R2_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(DirectPlayClue.game_actions(&pov).is_empty());
    }

    #[test]
    fn generates_clue_when_focus_is_the_only_card_and_it_is_playable() {
        // Player 1 has only R1 (card 10). With empty stacks R1 is playable.
        // Both the red clue and the "1" clue touch it — after dedup one action is returned.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // R1
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = DirectPlayClue.game_actions(&pov);

        assert_eq!(
            actions,
            vec![
                GameAction::Clue {
                    player_index: 1,
                    touched_card_deck_indexes: vec![10],
                    clue: Clue {
                        clue_type: 0,
                        clue_value: 0,
                    },
                },
                GameAction::Clue {
                    player_index: 1,
                    touched_card_deck_indexes: vec![10],
                    clue: Clue {
                        clue_type: 1,
                        clue_value: 1,
                    },
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
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // oldest = R2
        table_state.update_with_draw_action(20); // newest = R1
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R2_MASK), (20, R1_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = DirectPlayClue.game_actions(&pov);

        // Only the "1" clue action (touches [20] only)
        assert_eq!(
            actions,
            vec![GameAction::Clue {
                player_index: 1,
                touched_card_deck_indexes: vec![20],
                clue: Clue {
                    clue_type: 1,
                    clue_value: 1,
                },
            }]
        );
    }

    #[test]
    fn generates_clues_for_multiple_target_players() {
        // Player 1 has R1 (card 10), player 2 has Y1 (card 20). Both playable.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // R1
        table_state.player_on_turn_index = 2;
        table_state.update_with_draw_action(20); // Y1
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK), (20, Y1_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = DirectPlayClue.game_actions(&pov);

        assert!(actions.contains(&GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: vec![10],
            clue: Clue {
                clue_type: 0,
                clue_value: 0,
            },
        }));
        assert_eq!(actions, vec![
            GameAction::Clue {
                player_index: 1,
                touched_card_deck_indexes: vec![10],
                clue: Clue {
                    clue_type: 0,
                    clue_value: 0,
                },
            },
            GameAction::Clue {
                player_index: 1,
                touched_card_deck_indexes: vec![10],
                clue: Clue {
                    clue_type: 1,
                    clue_value: 1,
                },
            },
            GameAction::Clue {
                player_index: 2,
                touched_card_deck_indexes: vec![20],
                clue: Clue {
                    clue_type: 0,
                    clue_value: 1,
                },
            },
            GameAction::Clue {
                player_index: 2,
                touched_card_deck_indexes: vec![20],
                clue: Clue {
                    clue_type: 1,
                    clue_value: 1,
                },
            },
        ]);
    }

    // ── matches_action ─────────────────────────────────────────────────────────

    #[test]
    fn matches_action_true_when_focus_is_playable() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // R1
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let clue = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: vec![10],
            clue: Clue {
                clue_type: 0,
                clue_value: 0,
            },
        };
        assert!(DirectPlayClue.matches_action(&clue, &pov));
    }

    #[test]
    fn matches_action_false_when_focus_is_not_playable() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // R2
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R2_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let clue = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: vec![10],
            clue: Clue {
                clue_type: 0,
                clue_value: 0,
            },
        };
        assert!(!DirectPlayClue.matches_action(&clue, &pov));
    }

    #[test]
    fn matches_action_false_for_non_clue_action() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let table_state = initial_five_players_table_state();
        let knowledge = PlayerKnowledgeState::new(0);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let play = GameAction::Play {
            player_index: 0,
            card_deck_index: 5,
        };
        assert!(!DirectPlayClue.matches_action(&play, &pov));
    }

    #[test]
    fn matches_action_false_when_touched_is_empty_and_no_focus() {
        // An empty touched list → get_clue_focus_index returns None → false.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let table_state = initial_five_players_table_state();
        let knowledge = PlayerKnowledgeState::new(0);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let clue = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: vec![],
            clue: Clue {
                clue_type: 0,
                clue_value: 0,
            },
        };
        assert!(!DirectPlayClue.matches_action(&clue, &pov));
    }
}
