use crate::engine::convention::convention_tech::ClueTech;
use crate::engine::convention::hgroup::h_group_core::{
    get_chop_index, touched_cards_for_clue,
};
use crate::engine::convention::hgroup::h_group_tech::{priority, HGroupClueTech, SaveClueTech};
use crate::engine::game_state_snapshot::GameStateSnapshot;
use crate::engine::knowledge::knowledge_update::KnowledgeUpdate;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::card::CardDeckIndex;
use crate::game::clue::Clue;
use crate::game::clue_type::ClueType;
use crate::game::state::PlayerIndex;
use crate::impl_convention_tech_for_hgroup_clue_tech;

fn can_be_critical_saved(target_player_index: PlayerIndex, pov: &dyn PlayerPOV) -> bool {
    let chop_card = match get_chop_index(target_player_index, pov) {
        Some(c) => c,
        None => return false,
    };
    let card_id = match pov.card_identity(chop_card) {
        Some(id) => id,
        None => return false,
    };
    if !pov.is_critical_card_id(card_id) {
        return false;
    }
    if pov.static_data().variant.is_stack_ending_card(card_id) {
        return false;
    }
    if let Some(away_value) = pov.away_value(card_id) {
        away_value > 0
    } else {
        false
    }
}

fn critical_save_actions(
    player_on_turn_pov: &dyn PlayerPOV,
    clue_type: ClueType,
    clue_value_for_id: impl Fn(usize) -> u8,
) -> Vec<GameAction> {
    let active = player_on_turn_pov.player_on_turn_index();
    let num_players = player_on_turn_pov.static_data().number_of_players as usize;

    (0..num_players)
        .filter(|&p| p != active)
        .filter(|&target| can_be_critical_saved(target, player_on_turn_pov))
        .map(|target| {
            let chop_card_deck_index = get_chop_index(target, player_on_turn_pov).unwrap();
            let chop_variant_card_id = player_on_turn_pov
                .card_identity(chop_card_deck_index)
                .unwrap();
            let clue = Clue {
                clue_type,
                clue_value: clue_value_for_id(chop_variant_card_id),
            };
            let touched = touched_cards_for_clue(target, &clue, player_on_turn_pov);
            GameAction::Clue {
                player_index: target,
                touched_card_deck_indexes: touched,
                clue,
                turn: None,
            }
        })
        .collect()
}

fn critical_save_knowledge_updates(
    receiver: PlayerIndex,
    touched_card_deck_indexes: &[CardDeckIndex],
    clue: &Clue,
    snapshot: Option<&GameStateSnapshot>,
    player_pov: &dyn PlayerPOV,
) -> Vec<KnowledgeUpdate> {
    // Reconstruct the giver's POV as it was before the clue was applied so that the chop
    // is computed against the pre-clue hand state (before any card was marked as touched).
    let giver_pov_holder;
    let giver_pov: &dyn PlayerPOV = match snapshot {
        Some(snap) => {
            let giver = snap.table_state.player_on_turn_index;
            giver_pov_holder = snap.player_pov(giver, player_pov.static_data());
            &giver_pov_holder
        }
        None => player_pov,
    };
    let chop = match get_chop_index(receiver, giver_pov) {
        Some(c) if touched_card_deck_indexes.contains(&c) => c,
        _ => return vec![],
    };
    let static_data = giver_pov.static_data();
    let stacks_size = static_data.variant.stacks_size as usize;
    let total_ids = static_data.variant.number_of_suits as usize * stacks_size;
    let clue_mask = static_data.variant.empathy_for_clue(clue).as_bits();
    let mask: u64 = (0..total_ids)
        .filter(|&id| {
            if static_data.variant.is_stack_ending_card(id) {
                return false;
            }
            if (1u64 << id) & clue_mask == 0 {
                return false;
            }
            giver_pov.is_critical_card_id(id)
        })
        .fold(0u64, |acc, id| acc | (1 << id));
    if mask == 0 {
        return vec![];
    }
    vec![KnowledgeUpdate::NarrowPossibilities {
        card_deck_index: chop,
        mask,
    }]
}

fn critical_save_matches(
    player_index: PlayerIndex,
    touched_card_deck_indexes: &[CardDeckIndex],
    clue: &Clue,
    snapshot: Option<&GameStateSnapshot>,
    player_pov: &dyn PlayerPOV,
    clue_type: ClueType,
) -> bool {
    if clue.clue_type != clue_type {
        return false;
    }
    // Reconstruct the giver's POV from the pre-clue snapshot so that chop/focus are computed
    // against the hand state before this clue's touch flags were applied.
    let giver_pov_holder;
    let giver_pov: &dyn PlayerPOV = match snapshot {
        Some(snap) => {
            let giver = snap.table_state.player_on_turn_index;
            giver_pov_holder = snap.player_pov(giver, player_pov.static_data());
            &giver_pov_holder
        }
        None => player_pov,
    };
    can_be_critical_saved(player_index, giver_pov)
        && get_chop_index(player_index, giver_pov)
            .map(|chop| touched_card_deck_indexes.contains(&chop))
            .unwrap_or(false)
}

/// Save a critical card on chop by cluing its color (suit).
pub struct ColorCriticalSave;

impl ClueTech for ColorCriticalSave {
    fn clue_game_actions(&self, player_on_turn_pov: &dyn PlayerPOV) -> Vec<GameAction> {
        let stacks_size = player_on_turn_pov.static_data().variant.stacks_size as usize;
        critical_save_actions(player_on_turn_pov, ClueType::Color, |card_id| {
            (card_id / stacks_size) as u8
        })
    }

    fn matches_clue(
        &self,
        player_index: PlayerIndex,
        touched: &[CardDeckIndex],
        clue: &Clue,
        snapshot: Option<&GameStateSnapshot>,
        pov: &dyn PlayerPOV,
    ) -> bool {
        critical_save_matches(player_index, touched, clue, snapshot, pov, ClueType::Color)
    }

    fn clue_knowledge_updates(
        &self,
        player_index: PlayerIndex,
        touched: &[CardDeckIndex],
        clue: &Clue,
        snapshot: Option<&GameStateSnapshot>,
        player_pov: &dyn PlayerPOV,
    ) -> Vec<KnowledgeUpdate> {
        critical_save_knowledge_updates(player_index, touched, clue, snapshot, player_pov)
    }
}

impl SaveClueTech for ColorCriticalSave {}
impl HGroupClueTech for ColorCriticalSave {}
impl_convention_tech_for_hgroup_clue_tech!(ColorCriticalSave, priority::SAVE);

/// Save a critical card on chop by cluing its rank.
pub struct RankCriticalSave;

impl ClueTech for RankCriticalSave {
    fn clue_game_actions(&self, player_on_turn_pov: &dyn PlayerPOV) -> Vec<GameAction> {
        let variant = &player_on_turn_pov.static_data().variant;
        critical_save_actions(player_on_turn_pov, ClueType::Rank, |card_id| {
            variant.rank_of(card_id)
        })
    }

    fn matches_clue(
        &self,
        player_index: PlayerIndex,
        touched: &[CardDeckIndex],
        clue: &Clue,
        snapshot: Option<&GameStateSnapshot>,
        pov: &dyn PlayerPOV,
    ) -> bool {
        critical_save_matches(player_index, touched, clue, snapshot, pov, ClueType::Rank)
    }

    fn clue_knowledge_updates(
        &self,
        player_index: PlayerIndex,
        touched: &[CardDeckIndex],
        clue: &Clue,
        snapshot: Option<&GameStateSnapshot>,
        player_pov: &dyn PlayerPOV,
    ) -> Vec<KnowledgeUpdate> {
        critical_save_knowledge_updates(player_index, touched, clue, snapshot, player_pov)
    }
}

impl SaveClueTech for RankCriticalSave {}
impl HGroupClueTech for RankCriticalSave {}
impl_convention_tech_for_hgroup_clue_tech!(RankCriticalSave, priority::SAVE);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::convention::convention_tech::ConventionTech;
    use crate::engine::knowledge::player_knowledge_state::PlayerKnowledgeState;
    use crate::engine::knowledge::player_knowledge_state::knowledge_with_visible;
    use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
    use crate::engine::knowledge::team_knowledge::TeamKnowledge;
    use crate::game::deck::unit_test_constants::novariant_constants::{R2_MASK, R4_MASK, Y4_MASK};
    use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
        NOVAR_5_PLAYERS_STATIC_GAME_DATA, initial_five_players_table_state,
    };

    /// R4 has 2 copies total; discard one to make it critical.
    fn setup_with_critical_chop(
        card_id: usize,
        card_mask: u64,
    ) -> (
        crate::game::state::table_state::TableState,
        PlayerKnowledgeState,
    ) {
        let _static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
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
        let pov = LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = ColorCriticalSave.game_actions(&pov);

        assert_eq!(actions.len(), 1);
        assert!(
            matches!(&actions[0], GameAction::Clue { clue, .. } if clue.clue_type == ClueType::Color && clue.clue_value == 0)
        );
    }

    #[test]
    fn rank_save_generates_clue_when_chop_is_critical() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let (table_state, knowledge) = setup_with_critical_chop(3, R4_MASK); // R4 = rank 4
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = RankCriticalSave.game_actions(&pov);

        assert_eq!(actions.len(), 1);
        assert!(
            matches!(&actions[0], GameAction::Clue { clue, .. } if clue.clue_type == ClueType::Rank && clue.clue_value == 4)
        );
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
        let pov = LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

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
        let pov = LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

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
        let pov = LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = ColorCriticalSave.game_actions(&pov);

        assert_eq!(actions.len(), 1);
        let touched = match &actions[0] {
            GameAction::Clue {
                touched_card_deck_indexes,
                ..
            } => touched_card_deck_indexes,
            _ => panic!(),
        };
        assert!(touched.contains(&10));
        assert!(touched.contains(&20));
    }

    #[test]
    fn matches_action_true_for_color_save_when_chop_is_critical() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let (table_state, knowledge) = setup_with_critical_chop(3, R4_MASK);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        *team_knowledge.player_mut(0) = knowledge.clone();
        let pov = LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let action = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: smallvec::smallvec![10],
            clue: Clue {
                clue_type: ClueType::Color,
                clue_value: 0,
            },
            turn: None,
        };
        assert!(ColorCriticalSave.matches_action(&action, None, &pov));
    }

    #[test]
    fn matches_action_false_when_chop_is_not_critical() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10);
        table_state.player_on_turn_index = 0;
        let knowledge = knowledge_with_visible(0, &[(10, Y4_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        *team_knowledge.player_mut(0) = knowledge.clone();
        let pov = LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let action = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: smallvec::smallvec![10],
            clue: Clue {
                clue_type: ClueType::Color,
                clue_value: 1,
            },
            turn: None,
        };
        assert!(!ColorCriticalSave.matches_action(&action, None, &pov));
    }
}
