use crate::engine::convention::convention_tech::ConventionTech;
use crate::engine::convention::hgroup::h_group_core::{get_chop_index, get_clue_focus_index, touched_cards_for_clue};
use crate::engine::knowledge::knowledge_update::KnowledgeUpdate;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::card::{CardDeckIndex, VariantCardId};
use crate::game::clue::Clue;
use crate::game::variant::RANK_CLUE_TYPE;

/// Clue rank 2 to a teammate whose chop card is a 2 that is not immediately playable and cannot
/// be safely discarded (no other player has a copy outside their own chop).
pub struct TwoSave;

const RANK_2_CLUE: Clue = Clue { clue_type: RANK_CLUE_TYPE as u8, clue_value: 2 };

impl TwoSave {
    fn is_rank_two(card_id: VariantCardId, pov: &dyn PlayerPOV) -> bool {
        let rank2_mask = pov.static_data().variant.empathy_for_clue(&RANK_2_CLUE);
        (1u64 << card_id) & rank2_mask != 0
    }

    /// Returns true if the 2-save is legal: no other player has a copy of `card_id` that is
    /// NOT on their own chop (which would mean they already hold it safely).
    fn can_be_two_saved(card_id: VariantCardId, target: usize, pov: &dyn PlayerPOV) -> bool {
        let num_players = pov.static_data().number_of_players as usize;
        (0..num_players).filter(|&p| p != target).all(|p| {
            let has_copy = pov.table_state().hands[p]
                .cards()
                .iter()
                .any(|&idx| pov.card_identity(idx) == Some(card_id));
            if !has_copy {
                return true;
            }
            // They have a copy — it must be on their chop
            get_chop_index(p, pov)
                .map(|chop| pov.card_identity(chop) == Some(card_id))
                .unwrap_or(false)
        })
    }

    fn is_two_saveable(card_id: VariantCardId, target: usize, pov: &dyn PlayerPOV) -> bool {
        pov.away_value(card_id) > 0
            && Self::is_rank_two(card_id, pov)
            && Self::can_be_two_saved(card_id, target, pov)
    }
}

impl ConventionTech for TwoSave {
    fn priority(&self) -> u8 {
        0
    }

    fn game_actions(&self, pov: &dyn PlayerPOV) -> Vec<GameAction> {
        let active = pov.player_on_turn_index();
        let num_players = pov.static_data().number_of_players as usize;

        (0..num_players)
            .filter(|&p| p != active)
            .filter_map(|target| {
                let chop = get_chop_index(target, pov)?;
                let card_id = pov.card_identity(chop)?;
                if !Self::is_two_saveable(card_id, target, pov) {
                    return None;
                }
                let touched = touched_cards_for_clue(target, &RANK_2_CLUE, pov);
                Some(GameAction::Clue {
                    player_index: target,
                    touched_card_deck_indexes: touched,
                    clue: RANK_2_CLUE,
                })
            })
            .collect()
    }

    fn matches_action(&self, action: &GameAction, actor_pov: &dyn PlayerPOV) -> bool {
        if let GameAction::Clue { player_index, clue, .. } = action {
            *clue == RANK_2_CLUE
                && get_chop_index(*player_index, actor_pov)
                    .and_then(|chop| actor_pov.card_identity(chop))
                    .map(|card_id| Self::is_two_saveable(card_id, *player_index, actor_pov))
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
        let rank2_mask = static_data.variant.empathy_for_clue(&RANK_2_CLUE);
        let mask: u64 = (0..total_ids)
            .filter(|&id| (1u64 << id) & rank2_mask != 0 && player_pov.away_value(id) > 0)
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
    use crate::game::deck::unit_test_constant::novariant_constants::NoVarCards::*;
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
    fn generates_clue_when_chop_is_a_non_playable_2() {
        // Stacks empty → R2 is 1-away (not playable). No other player has R2 → can be saved.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // R2
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R2_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = TwoSave.game_actions(&pov);
        assert_eq!(actions, vec![GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: vec![10],
            clue: RANK_2_CLUE,
        }]);
    }

    #[test]
    fn returns_empty_when_chop_2_is_immediately_playable() {
        // R1 played → R2 is now playable (away=0). Should not two-save it.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_play_action_of_specific_card(0, R1.as_variant_card_id(), &static_data);
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // R2, now playable
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R2_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(TwoSave.game_actions(&pov).is_empty());
    }

    #[test]
    fn returns_empty_when_chop_is_not_a_2() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // R3
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R3_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(TwoSave.game_actions(&pov).is_empty());
    }

    #[test]
    fn returns_empty_when_another_player_has_copy_not_on_chop() {
        // Player 2 has R2 not on chop → R2 is already safe → no two-save needed.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // R2 on chop of player 1
        table_state.player_on_turn_index = 2;
        table_state.update_with_draw_action(20); // older card (chop of player 2)
        table_state.update_with_draw_action(30); // R2 not on chop of player 2
        table_state.player_on_turn_index = 0;

        // R2 id = 1; player 2's chop = card 20, not card 30
        let knowledge = knowledge_with_visible(0, &[(10, R2_MASK), (20, Y1_MASK), (30, R2_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(TwoSave.game_actions(&pov).is_empty());
    }

    #[test]
    fn touches_all_2s_in_hand_when_cluing() {
        // Player 1 has R2 on chop (card 10) and Y2 newer (card 20).
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // oldest = R2 (chop)
        table_state.update_with_draw_action(20); // newest = Y2
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R2_MASK), (20, Y2_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = TwoSave.game_actions(&pov);
        assert_eq!(actions.len(), 1);
        let touched = match &actions[0] {
            GameAction::Clue { touched_card_deck_indexes, .. } => touched_card_deck_indexes,
            _ => panic!(),
        };
        assert!(touched.contains(&10));
        assert!(touched.contains(&20));
    }

    // ── matches_action ─────────────────────────────────────────────────────────

    #[test]
    fn matches_action_true_when_chop_is_saveable_2() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10);
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R2_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let action = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: vec![10],
            clue: RANK_2_CLUE,
        };
        assert!(TwoSave.matches_action(&action, &pov));
    }

    #[test]
    fn matches_action_false_when_chop_2_is_playable() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_play_action_of_specific_card(0, R1.as_variant_card_id(), &static_data);
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // R2 now playable
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R2_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let action = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: vec![10],
            clue: RANK_2_CLUE,
        };
        assert!(!TwoSave.matches_action(&action, &pov));
    }

    #[test]
    fn matches_action_false_for_non_clue_action() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let table_state = initial_five_players_table_state();
        let knowledge = PlayerKnowledgeState::new(0);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(!TwoSave.matches_action(&GameAction::Play { player_index: 0, card_deck_index: 5 }, &pov));
    }

    // ── knowledge_updates ──────────────────────────────────────────────────────

    #[test]
    fn knowledge_updates_narrows_focus_to_non_playable_rank_2s() {
        // Player 0 receives a rank-2 clue. Card 10 is touched (focus = chop).
        // Stacks empty → all 2s have away=1 (not playable). Mask should be all rank-2 IDs.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_draw_action(10);
        table_state.clue_touched_cards |= 1 << 10;

        let knowledge = knowledge_with_visible(0, &[]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let updates = TwoSave.knowledge_updates(&pov);
        assert_eq!(updates.len(), 1);
        if let KnowledgeUpdate::NarrowPossibilities { card_deck_index, mask } = &updates[0] {
            assert_eq!(*card_deck_index, 10);
            // All rank-2 IDs (R2=1, Y2=6, G2=11, B2=16, P2=21) should be in the mask.
            assert!(mask & R2_MASK != 0);
            assert!(mask & Y2_MASK != 0);
            // Rank-1 IDs should not be in the mask.
            assert!(mask & R1_MASK == 0);
        } else {
            panic!("expected NarrowPossibilities");
        }
    }

    #[test]
    fn knowledge_updates_excludes_playable_2s_from_mask() {
        // R1 played → R2 is now playable (away=0). R2 must be excluded from the mask.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.update_with_play_action_of_specific_card(0, R1.as_variant_card_id(), &static_data);
        table_state.player_on_turn_index = 0;
        table_state.update_with_draw_action(10);
        table_state.clue_touched_cards |= 1 << 10;

        let knowledge = knowledge_with_visible(0, &[]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let updates = TwoSave.knowledge_updates(&pov);
        assert_eq!(updates.len(), 1);
        if let KnowledgeUpdate::NarrowPossibilities { mask, .. } = &updates[0] {
            assert!(mask & R2_MASK == 0, "R2 is playable, must not be in mask");
            assert!(mask & Y2_MASK != 0, "Y2 is still 1-away, must be in mask");
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
        let knowledge = knowledge_with_visible(0, &[]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(TwoSave.knowledge_updates(&pov).is_empty());
    }

    #[test]
    fn knowledge_updates_returns_empty_when_all_2s_are_playable() {
        // All 5 suits have R1..P1 played → all 2s are playable → mask = 0 → empty.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        for &card_id in &[R1, Y1, G1, B1, P1] {
            table_state.update_with_play_action_of_specific_card(0, card_id as usize, &static_data);
        }
        table_state.player_on_turn_index = 0;
        table_state.update_with_draw_action(10);
        table_state.clue_touched_cards |= 1 << 10;

        let knowledge = knowledge_with_visible(0, &[]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(TwoSave.knowledge_updates(&pov).is_empty());
    }
}
