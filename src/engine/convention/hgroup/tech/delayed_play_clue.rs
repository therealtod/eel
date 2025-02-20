use crate::engine::convention::convention_tech::ConventionTech;
use crate::engine::convention::hgroup::h_group_core::{clues_for_player_with_focus, get_clue_focus_index};
use crate::engine::knowledge::knowledge_update::KnowledgeUpdate;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::card::{CardDeckIndex, VariantCardId};

/// Give a clue whose focus card is not immediately playable, but will become playable once all
/// connecting cards (which are already visible to the team) are played.
pub struct DelayedPlayClue;

impl DelayedPlayClue {
    /// Returns true if every card that must be played before `card_id` is already visible
    /// (known identity) in some teammate's hand.
    ///
    /// Connecting cards are those with IDs `card_id - away + 1` through `card_id - 1`, where
    /// `away = away_value(card_id)`. Within a suit, card IDs are consecutive.
    fn connecting_cards_are_known(card_id: VariantCardId, pov: &dyn PlayerPOV) -> bool {
        let away = pov.away_value(card_id) as usize;
        let active = pov.player_on_turn_index();
        let num_players = pov.static_data().number_of_players as usize;

        (1..away).all(|offset| {
            let connecting_id = card_id - offset;
            (0..num_players)
                .filter(|&p| p != active)
                .any(|p| {
                    pov.table_state().hands[p]
                        .cards()
                        .iter()
                        .any(|&idx| pov.card_identity(idx) == Some(connecting_id))
                })
        })
    }
}

impl ConventionTech for DelayedPlayClue {
    fn priority(&self) -> u8 {
        1
    }

    fn game_actions(&self, pov: &dyn PlayerPOV) -> Vec<GameAction> {
        let active = pov.player_on_turn_index();
        let num_players = pov.static_data().number_of_players as usize;

        (0..num_players)
            .filter(|&p| p != active)
            .flat_map(|target| {
                clues_for_player_with_focus(target, pov)
                    .into_iter()
                    .filter_map(|(action, focus_idx)| {
                        let card_id = pov.card_identity(focus_idx)?;
                        let away = pov.away_value(card_id);
                        if away > 0 && Self::connecting_cards_are_known(card_id, pov) {
                            Some(action)
                        } else {
                            None
                        }
                    })
            })
            .collect()
    }

    fn matches_action(&self, action: &GameAction, actor_pov: &dyn PlayerPOV) -> bool {
        if let GameAction::Clue { player_index, touched_card_deck_indexes, .. } = action {
            get_clue_focus_index(*player_index, touched_card_deck_indexes, actor_pov)
                .and_then(|focus| actor_pov.card_identity(focus))
                .map(|card_id| {
                    let away = actor_pov.away_value(card_id);
                    away > 0 && Self::connecting_cards_are_known(card_id, actor_pov)
                })
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
        let static_data = player_pov.static_data();
        let total_ids = static_data.variant.number_of_suits as usize * static_data.variant.stacks_size as usize;
        let mask: u64 = (0..total_ids)
            .filter(|&id| {
                let away = player_pov.away_value(id);
                away > 0 && Self::connecting_cards_are_known(id, player_pov)
            })
            .fold(0u64, |acc, id| acc | (1 << id));
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
    use crate::game::clue::Clue;
    use crate::game::deck::unit_test_constant::novariant_constants::*;
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

    // ── game_actions ───────────────────────────────────────────────────────────

    #[test]
    fn game_actions_returns_empty_when_no_connecting_card_is_visible() {
        // Player 1 has R3 (2-away). Connecting card R2 is not visible anywhere.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // R3
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R3_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(DelayedPlayClue.game_actions(&pov).is_empty());
    }

    #[test]
    fn game_actions_returns_empty_when_focus_is_immediately_playable() {
        // Player 1 has R1 (away=0). DirectPlayClue handles this; DelayedPlayClue should skip it.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // R1
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(DelayedPlayClue.game_actions(&pov).is_empty());
    }

    #[test]
    fn game_actions_generates_clue_when_connecting_card_is_visible_in_teammate_hand() {
        // Player 2 has R2 (card 20, visible to player 0). Player 1 has R3 (card 10).
        // R3 is 2-away; connecting card R2 is visible in player 2's hand → delayed play clue for player 1.
        // R2 is 1-away with no connecting cards needed → also a delayed play clue for player 2.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // R3
        table_state.player_on_turn_index = 2;
        table_state.update_with_draw_action(20); // R2
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R3_MASK), (20, R2_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = DelayedPlayClue.game_actions(&pov);
        assert!(actions.iter().any(|a| matches!(a, GameAction::Clue { player_index: 1, .. })));
        assert!(actions.iter().any(|a| matches!(a, GameAction::Clue { player_index: 2, .. })));
    }

    #[test]
    fn game_actions_does_not_clue_own_player() {
        // Even if player 0 has a delayed-playable card, we never clue ourselves.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 0;
        table_state.update_with_draw_action(10); // R3 in own hand
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(20); // R2 visible
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R3_MASK), (20, R2_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = DelayedPlayClue.game_actions(&pov);
        assert!(actions.iter().all(|a| !matches!(a, GameAction::Clue { player_index: 0, .. })));
    }

    // ── matches_action ─────────────────────────────────────────────────────────

    #[test]
    fn matches_action_true_when_focus_is_delayed_playable_with_connecting_card_visible() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // R3
        table_state.player_on_turn_index = 2;
        table_state.update_with_draw_action(20); // R2
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R3_MASK), (20, R2_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let clue = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: vec![10],
            clue: Clue { clue_type: 0, clue_value: 0 }, // red clue
        };
        assert!(DelayedPlayClue.matches_action(&clue, &pov));
    }

    #[test]
    fn matches_action_false_when_focus_is_immediately_playable() {
        // R1 has away=0, so it's not a delayed play clue.
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
            clue: Clue { clue_type: 1, clue_value: 1 },
        };
        assert!(!DelayedPlayClue.matches_action(&clue, &pov));
    }

    #[test]
    fn matches_action_false_when_connecting_card_not_visible() {
        // R3 is 2-away but R2 is not visible anywhere.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // R3
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R3_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let clue = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: vec![10],
            clue: Clue { clue_type: 0, clue_value: 0 },
        };
        assert!(!DelayedPlayClue.matches_action(&clue, &pov));
    }

    #[test]
    fn matches_action_false_for_non_clue_action() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let table_state = initial_five_players_table_state();
        let knowledge = PlayerKnowledgeState::new(0);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(!DelayedPlayClue.matches_action(&GameAction::Play { player_index: 0, card_deck_index: 5 }, &pov));
    }

    // ── knowledge_updates ──────────────────────────────────────────────────────

    #[test]
    fn knowledge_updates_narrows_focus_to_delayed_playable_ids() {
        // Player 0 is the receiver. They have R3 (card 10, touched).
        // Player 1 has R2 (card 20, visible) → R3 is a valid delayed play target.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(10); // R3 in player 0's hand
        table_state.clue_touched_cards |= 1 << 10;
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(20); // R2 in player 1's hand
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R3_MASK), (20, R2_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let updates = DelayedPlayClue.knowledge_updates(&pov);

        assert_eq!(updates.len(), 1);
        if let KnowledgeUpdate::NarrowPossibilities { card_deck_index, mask } = &updates[0] {
            assert_eq!(*card_deck_index, 10);
            // R3 (id=2) must be in the mask; R1 (id=0, away=0) must not be.
            assert!(mask & R3_MASK != 0, "R3 should be in the mask");
            assert!(mask & R1_MASK == 0, "R1 (immediately playable) should not be in the mask");
        } else {
            panic!("expected NarrowPossibilities");
        }
    }

    #[test]
    fn knowledge_updates_returns_empty_when_no_touched_cards() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(10);
        // No clue_touched_cards set.
        let knowledge = knowledge_with_visible(0, &[(10, R3_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(DelayedPlayClue.knowledge_updates(&pov).is_empty());
    }

    #[test]
    fn knowledge_updates_returns_empty_when_no_delayed_playable_ids_exist() {
        // All stacks are complete (size 5) → every card has away=0 → mask is 0 → empty updates.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        // Fill all 5 stacks to size 5 by advancing player_on_turn_index each play.
        use crate::game::deck::unit_test_constant::novariant_constants::NoVarCards::*;
        for &card_id in &[R1, R2, R3, R4, R5, Y1, Y2, Y3, Y4, Y5, G1, G2, G3, G4, G5, B1, B2, B3, B4, B5, P1, P2, P3, P4, P5] {
            table_state.update_with_play_action_of_specific_card(0, card_id as usize, &static_data);
        }
        table_state.update_with_draw_action(10);
        table_state.clue_touched_cards |= 1 << 10;

        let knowledge = knowledge_with_visible(0, &[(10, R3_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(DelayedPlayClue.knowledge_updates(&pov).is_empty());
    }
}