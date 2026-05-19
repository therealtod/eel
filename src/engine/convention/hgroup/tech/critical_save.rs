use crate::engine::convention::convention_tech::ClueTech;
use crate::engine::convention::hgroup::h_group_core::{get_chop_index, touched_cards_for_clue};
use crate::engine::convention::hgroup::h_group_tech::{HGroupClueTech, SaveClueTech, priority};
use crate::engine::game_state_snapshot::GameStateSnapshot;
use crate::engine::knowledge::knowledge_update::{Hypothesis, KnowledgeUpdate};
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::card::{CardDeckIndex, VariantCardId};
use crate::game::clue::Clue;
use crate::game::clue_type::ClueType;
use crate::game::state::PlayerIndex;
use crate::impl_convention_tech_for_hgroup_clue_tech;

/// Predicate on a known card identity: would saving a card of this identity on chop be a valid
/// critical save from `pov`'s perspective?
fn is_critical_save_id(card_id: VariantCardId, pov: &dyn PlayerPOV) -> bool {
    let variant = &pov.static_data().variant;
    pov.is_critical_card_id(card_id)
        && variant.rank_of(card_id) != 5
        && !variant.is_stack_ending_card(card_id)
        && pov.away_value(card_id).is_some_and(|a| a > 0)
}

fn can_be_critical_saved(target_player_index: PlayerIndex, observer_pov: &dyn PlayerPOV) -> bool {
    let Some(chop_card) = get_chop_index(target_player_index, observer_pov) else {
        return false;
    };
    let Some(card_id) = observer_pov.card_identity(chop_card) else {
        return false;
    };
    is_critical_save_id(card_id, observer_pov)
}

fn critical_save_actions(
    active_player_pov: &dyn PlayerPOV,
    clue_type: ClueType,
    clue_value_for_id: impl Fn(usize) -> u8,
) -> Vec<GameAction> {
    let active_player_index = active_player_pov.active_player_index();
    let num_players = active_player_pov.static_data().number_of_players as usize;

    (0..num_players)
        .filter(|&player_index| player_index != active_player_index)
        .filter(|&target_player_index| {
            can_be_critical_saved(target_player_index, active_player_pov)
        })
        .map(|target_player_index| {
            let chop_card_deck_index =
                get_chop_index(target_player_index, active_player_pov).unwrap();
            let chop_variant_card_id = active_player_pov
                .card_identity(chop_card_deck_index)
                .unwrap();
            let clue = Clue {
                clue_type,
                clue_value: clue_value_for_id(chop_variant_card_id),
            };
            let touched_card_deck_indexes =
                touched_cards_for_clue(target_player_index, &clue, active_player_pov);
            GameAction::Clue {
                player_index: target_player_index,
                touched_card_deck_indexes,
                clue,
                turn: active_player_pov.table_state().current_turn,
            }
        })
        .collect()
}

fn critical_save_knowledge_updates(
    receiver: PlayerIndex,
    touched_card_deck_indexes: &[CardDeckIndex],
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
    let chop = match get_chop_index(receiver, &giver_pov) {
        Some(c) if touched_card_deck_indexes.contains(&c) => c,
        _ => return Hypothesis::empty(),
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
        return Hypothesis::empty();
    }
    Hypothesis::unconditional(vec![KnowledgeUpdate::NarrowPossibilities {
        card_deck_index: chop,
        mask,
    }])
}

fn critical_save_matches(
    player_index: PlayerIndex,
    touched_card_deck_indexes: &[CardDeckIndex],
    clue: &Clue,
    turn: usize,
    history: &[GameStateSnapshot],
    observer_pov: &dyn PlayerPOV,
    clue_type: ClueType,
) -> bool {
    if clue.clue_type != clue_type {
        return false;
    }
    let Some(snap) = history.get(turn) else {
        return false;
    };
    let giver = snap.table_state.active_player_index;
    let giver_pov = snap.player_pov(giver, observer_pov.static_data());
    let Some(chop) = get_chop_index(player_index, &giver_pov) else {
        return false;
    };
    if !touched_card_deck_indexes.contains(&chop) {
        return false;
    }
    // Match if any chop identity consistent with the observer's empathy and the clue mask would
    // have constituted a critical save from the giver's view. Receiver: wide empathy → genuine
    // existential. Others: singleton empathy → collapses to the previous direct check.
    let static_data = observer_pov.static_data();
    let total_ids =
        static_data.variant.number_of_suits as usize * static_data.variant.stacks_size as usize;
    let clue_mask = static_data.variant.empathy_for_clue(clue).as_bits();
    let candidates = observer_pov.inferred_identities(chop).as_bits() & clue_mask;
    (0..total_ids).any(|id| (candidates & (1u64 << id)) != 0 && is_critical_save_id(id, &giver_pov))
}

/// Save a critical card on chop by cluing its color (suit) or rank.
#[derive(Clone, Copy)]
pub struct CriticalSave(pub ClueType);

#[allow(non_upper_case_globals)]
pub const ColorCriticalSave: CriticalSave = CriticalSave(ClueType::Color);
#[allow(non_upper_case_globals)]
pub const RankCriticalSave: CriticalSave = CriticalSave(ClueType::Rank);

impl ClueTech for CriticalSave {
    fn clue_game_actions(&self, pov: &dyn PlayerPOV) -> Vec<GameAction> {
        match self.0 {
            ClueType::Color => {
                let stacks_size = pov.static_data().variant.stacks_size as usize;
                critical_save_actions(pov, ClueType::Color, move |card_id| {
                    (card_id / stacks_size) as u8
                })
            }
            ClueType::Rank => {
                let variant = &pov.static_data().variant;
                critical_save_actions(pov, ClueType::Rank, |card_id| variant.rank_of(card_id))
            }
        }
    }

    fn matches_clue(
        &self,
        player_index: PlayerIndex,
        touched: &[CardDeckIndex],
        clue: &Clue,
        turn: usize,
        history: &[GameStateSnapshot],
        pov: &dyn PlayerPOV,
    ) -> bool {
        critical_save_matches(player_index, touched, clue, turn, history, pov, self.0)
    }

    fn clue_knowledge_updates(
        &self,
        player_index: PlayerIndex,
        touched: &[CardDeckIndex],
        clue: &Clue,
        turn: usize,
        history: &[GameStateSnapshot],
        player_pov: &dyn PlayerPOV,
    ) -> Hypothesis {
        critical_save_knowledge_updates(player_index, touched, clue, turn, history, player_pov)
    }
}

impl SaveClueTech for CriticalSave {}
impl HGroupClueTech for CriticalSave {}
impl_convention_tech_for_hgroup_clue_tech!(CriticalSave, priority::SAVE);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::convention::convention_tech::ConventionTech;
    use crate::engine::game_state_snapshot::GameStateSnapshot;
    use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
    use crate::engine::knowledge::player_knowledge::PlayerKnowledge;
    use crate::engine::knowledge::player_knowledge::knowledge_with_visible;
    use crate::engine::knowledge::team_knowledge::TeamKnowledge;
    use crate::game::deck::unit_test_constants::novariant_constants::{R2_MASK, R4_MASK, Y4_MASK};
    use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
        NOVAR_5_PLAYERS_STATIC_GAME_DATA, initial_five_players_table_state,
    };

    /// R4 has 2 copies total; discard one to make it critical.
    fn setup_with_critical_chop(
        card_id: usize,
        card_mask: u64,
    ) -> (crate::game::state::table_state::TableState, PlayerKnowledge) {
        let _static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // chop
        table_state.active_player_index = 0;
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
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

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
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

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
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        table_state.active_player_index = 0;
        // No discards — R4 still has 2 copies, not critical
        let knowledge = knowledge_with_visible(0, &[(10, R4_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(ColorCriticalSave.game_actions(&pov).is_empty());
        assert!(RankCriticalSave.game_actions(&pov).is_empty());
    }

    #[test]
    fn returns_empty_when_chop_identity_unknown() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        table_state.active_player_index = 0;
        table_state.discard_pile.add_card_with_id(3);
        // Card 10 identity not known to player 0
        let knowledge = knowledge_with_visible(0, &[]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(ColorCriticalSave.game_actions(&pov).is_empty());
        assert!(RankCriticalSave.game_actions(&pov).is_empty());
    }

    #[test]
    fn color_save_touches_all_cards_of_same_suit() {
        // Player 1 has R4 (chop, critical) and R2 (newer).
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // oldest = R4 (chop)
        table_state.update_with_draw_action(20); // newest = R2
        table_state.active_player_index = 0;
        table_state.discard_pile.add_card_with_id(3); // discard one R4
        let knowledge = knowledge_with_visible(0, &[(10, R4_MASK), (20, R2_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

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
        let snapshot = GameStateSnapshot::new(table_state.clone(), team_knowledge.clone());
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let action = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: smallvec::smallvec![10],
            clue: Clue {
                clue_type: ClueType::Color,
                clue_value: 0,
            },
            turn: 0,
        };
        assert!(ColorCriticalSave.matches_action(&action, &[snapshot], &pov));
    }

    // ── empty-history behaviour ─────────────────────────────────────────────────

    #[test]
    fn matches_action_false_when_history_is_empty() {
        // Both ColorCriticalSave and RankCriticalSave safely return false with &[].
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        table_state.active_player_index = 0;
        table_state.discard_pile.add_card_with_id(3); // make R4 critical
        let knowledge = knowledge_with_visible(0, &[(10, R4_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let action = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: smallvec::smallvec![10],
            clue: Clue {
                clue_type: ClueType::Color,
                clue_value: 0,
            },
            turn: 0,
        };
        assert!(!ColorCriticalSave.matches_action(&action, &[], &pov));
        assert!(!RankCriticalSave.matches_action(&action, &[], &pov));
    }

    #[test]
    fn color_critical_save_knowledge_updates_returns_empty_when_history_is_empty() {
        // With no history we cannot reconstruct the giver's POV; the tech yields no claim.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let table_state = initial_five_players_table_state();
        let knowledge = knowledge_with_visible(0, &[]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let updates = ColorCriticalSave.knowledge_updates(
            &GameAction::Clue {
                player_index: 1,
                touched_card_deck_indexes: smallvec::smallvec![10],
                clue: Clue {
                    clue_type: ClueType::Color,
                    clue_value: 0,
                },
                turn: 0,
            },
            &[],
            &pov,
        );
        assert!(updates.is_empty());
    }

    #[test]
    fn matches_action_false_when_chop_is_not_critical() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        table_state.active_player_index = 0;
        let knowledge = knowledge_with_visible(0, &[(10, Y4_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        *team_knowledge.player_mut(0) = knowledge.clone();
        let snapshot = GameStateSnapshot::new(table_state.clone(), team_knowledge.clone());
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let action = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: smallvec::smallvec![10],
            clue: Clue {
                clue_type: ClueType::Color,
                clue_value: 1,
            },
            turn: 0,
        };
        assert!(!ColorCriticalSave.matches_action(&action, &[snapshot], &pov));
    }
}
