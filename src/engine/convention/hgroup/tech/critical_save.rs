use crate::engine::convention::convention_tech::ConventionTech;
use crate::engine::convention::hgroup::h_group_core::{get_chop_index, touched_cards_for_clue};
use crate::engine::knowledge::knowledge_update::KnowledgeUpdate;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::clue::Clue;
use crate::game::variant::RANK_CLUE_TYPE;

const COLOR_CLUE_TYPE: usize = 0;

fn critical_save_actions(
    player_on_turn_pov: &dyn PlayerPOV,
    clue_type: usize,
    clue_value_for_id: impl Fn(usize) -> u8,
) -> Vec<GameAction> {
    let active = player_on_turn_pov.player_on_turn_index();
    let static_data = player_on_turn_pov.static_data();
    let num_players = static_data.number_of_players as usize;

    (0..num_players)
        .filter(|&p| p != active)
        .filter_map(|target| {
            let chop = get_chop_index(target, player_on_turn_pov)?;
            if !player_on_turn_pov.is_critical(chop) {
                return None;
            }
            let chop_id = player_on_turn_pov.card_identity(chop)?;
            let stacks_size = static_data.variant.stacks_size as usize;
            if chop_id % stacks_size == stacks_size - 1 { return None; } // rank-5: use 5-save instead
            let clue = Clue { clue_type: clue_type as u8, clue_value: clue_value_for_id(chop_id) };
            let touched = touched_cards_for_clue(target, &clue, player_on_turn_pov);
            Some(GameAction::Clue {
                player_index: target,
                touched_card_deck_indexes: touched,
                clue,
            })
        })
        .collect()
}

fn critical_save_knowledge_updates(player_pov: &dyn PlayerPOV) -> Vec<KnowledgeUpdate> {
    let receiver = player_pov.player_on_turn_index();
    let chop = match get_chop_index(receiver, player_pov) {
        Some(c) => c,
        None => return vec![],
    };
    let static_data = player_pov.static_data();
    let table_state = player_pov.table_state();
    let stacks_size = static_data.variant.stacks_size as usize;
    let total_ids = static_data.variant.number_of_suits as usize * stacks_size;
    let mask: u64 = (0..total_ids)
        .filter(|&id| {
            if id % stacks_size == stacks_size - 1 { return false; } // exclude 5s
            let total = static_data.variant.card_copies_count_by_id[id];
            let discarded = table_state.discard_pile.copies_of(id);
            total > 0 && discarded == total - 1
        })
        .fold(0u64, |acc, id| acc | (1 << id));
    if mask == 0 { return vec![]; }
    vec![KnowledgeUpdate::NarrowPossibilities { card_deck_index: chop, mask }]
}

fn critical_save_matches(action: &GameAction, actor_pov: &dyn PlayerPOV, clue_type: u8) -> bool {
    let GameAction::Clue { player_index, clue, .. } = action else { return false };
    if clue.clue_type != clue_type { return false }
    let stacks_size = actor_pov.static_data().variant.stacks_size as usize;
    get_chop_index(*player_index, actor_pov)
        .map(|chop| {
            actor_pov.is_critical(chop)
                && actor_pov.card_identity(chop)
                    .map(|id| id % stacks_size != stacks_size - 1)
                    .unwrap_or(true)
        })
        .unwrap_or(false)
}

/// Save a critical card on chop by cluing its color (suit).
pub struct ColorCriticalSave;

impl ConventionTech for ColorCriticalSave {
    fn priority(&self) -> u8 { 0 }

    fn game_actions(&self, player_on_turn_pov: &dyn PlayerPOV) -> Vec<GameAction> {
        let stacks_size = player_on_turn_pov.static_data().variant.stacks_size as usize;
        critical_save_actions(player_on_turn_pov, COLOR_CLUE_TYPE, |card_id| {
            (card_id / stacks_size) as u8
        })
    }

    fn matches_action(&self, action: &GameAction, actor_pov: &dyn PlayerPOV) -> bool {
        critical_save_matches(action, actor_pov, COLOR_CLUE_TYPE as u8)
    }

    fn knowledge_updates(&self, player_pov: &dyn PlayerPOV) -> Vec<KnowledgeUpdate> {
        critical_save_knowledge_updates(player_pov)
    }
}

/// Save a critical card on chop by cluing its rank.
pub struct RankCriticalSave;

impl ConventionTech for RankCriticalSave {
    fn priority(&self) -> u8 { 0 }

    fn game_actions(&self, player_on_turn_pov: &dyn PlayerPOV) -> Vec<GameAction> {
        let stacks_size = player_on_turn_pov.static_data().variant.stacks_size as usize;
        critical_save_actions(player_on_turn_pov, RANK_CLUE_TYPE, |card_id| {
            // Rank is 1-based: position within suit + 1
            (card_id % stacks_size + 1) as u8
        })
    }

    fn matches_action(&self, action: &GameAction, actor_pov: &dyn PlayerPOV) -> bool {
        critical_save_matches(action, actor_pov, RANK_CLUE_TYPE as u8)
    }

    fn knowledge_updates(&self, player_pov: &dyn PlayerPOV) -> Vec<KnowledgeUpdate> {
        critical_save_knowledge_updates(player_pov)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::convention::convention_tech::ConventionTech;
    use crate::engine::knowledge::player_knowledge_state::PlayerKnowledgeState;
    use crate::engine::knowledge::team_knowledge::TeamKnowledge;
    use crate::engine::knowledge::player_pov_view::PlayerPOVView;
    use crate::game::deck::unit_test_constant::novariant_constants::{R2_MASK, R4_MASK, Y4_MASK};
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

    /// R4 has 2 copies total; discard one to make it critical.
    fn setup_with_critical_chop(card_id: usize, card_mask: u64) -> (
        crate::game::state::table_state::TableState,
        PlayerKnowledgeState,
    ) {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // chop
        table_state.player_on_turn_index = 0;
        // Discard one copy to make it critical (R4 has 2 copies, so 1 discarded = critical)
        table_state.discard_pile.add_card_with_id(card_id);
        let knowledge = knowledge_with_visible(0, &[(10, card_mask)]);
        (table_state, knowledge)
    }

    #[test]
    fn color_save_generates_clue_when_chop_is_critical() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let (table_state, knowledge) = setup_with_critical_chop(3, R4_MASK); // R4 = id 3
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = ColorCriticalSave.game_actions(&pov);

        assert_eq!(actions.len(), 1);
        assert!(matches!(&actions[0], GameAction::Clue { clue, .. } if clue.clue_type == 0 && clue.clue_value == 0));
    }

    #[test]
    fn rank_save_generates_clue_when_chop_is_critical() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let (table_state, knowledge) = setup_with_critical_chop(3, R4_MASK); // R4 = rank 4
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = RankCriticalSave.game_actions(&pov);

        assert_eq!(actions.len(), 1);
        assert!(matches!(&actions[0], GameAction::Clue { clue, .. } if clue.clue_type == 1 && clue.clue_value == 4));
    }

    #[test]
    fn returns_empty_when_chop_is_not_critical() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10);
        table_state.player_on_turn_index = 0;
        // No discards — R4 still has 2 copies, not critical
        let knowledge = knowledge_with_visible(0, &[(10, R4_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(ColorCriticalSave.game_actions(&pov).is_empty());
        assert!(RankCriticalSave.game_actions(&pov).is_empty());
    }

    #[test]
    fn returns_empty_when_chop_identity_unknown() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10);
        table_state.player_on_turn_index = 0;
        table_state.discard_pile.add_card_with_id(3);
        // Card 10 identity not known to player 0
        let knowledge = knowledge_with_visible(0, &[]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(ColorCriticalSave.game_actions(&pov).is_empty());
        assert!(RankCriticalSave.game_actions(&pov).is_empty());
    }

    #[test]
    fn color_save_touches_all_cards_of_same_suit() {
        // Player 1 has R4 (chop, critical) and R2 (newer).
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // oldest = R4 (chop)
        table_state.update_with_draw_action(20); // newest = R2
        table_state.player_on_turn_index = 0;
        table_state.discard_pile.add_card_with_id(3); // discard one R4
        let knowledge = knowledge_with_visible(0, &[(10, R4_MASK), (20, R2_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = ColorCriticalSave.game_actions(&pov);

        assert_eq!(actions.len(), 1);
        let touched = match &actions[0] {
            GameAction::Clue { touched_card_deck_indexes, .. } => touched_card_deck_indexes,
            _ => panic!(),
        };
        assert!(touched.contains(&10));
        assert!(touched.contains(&20));
    }

    #[test]
    fn matches_action_true_for_color_save_when_chop_is_critical() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let (table_state, knowledge) = setup_with_critical_chop(3, R4_MASK);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let action = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: vec![10],
            clue: Clue { clue_type: 0, clue_value: 0 },
        };
        assert!(ColorCriticalSave.matches_action(&action, &pov));
    }

    #[test]
    fn matches_action_false_when_chop_is_not_critical() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10);
        table_state.player_on_turn_index = 0;
        let knowledge = knowledge_with_visible(0, &[(10, Y4_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = PlayerPOVView::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let action = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: vec![10],
            clue: Clue { clue_type: 0, clue_value: 1 },
        };
        assert!(!ColorCriticalSave.matches_action(&action, &pov));
    }
}
