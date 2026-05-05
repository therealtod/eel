use crate::engine::convention::convention_tech::ClueTech;
use crate::engine::convention::hgroup::h_group_core::{
    get_chop_index, giver_pov, touched_cards_for_clue,
};
use crate::engine::convention::hgroup::h_group_tech::{priority, HGroupClueTech, SaveClueTech};
use crate::engine::knowledge::knowledge_update::KnowledgeUpdate;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::card::CardDeckIndex;
use crate::game::clue::Clue;
use crate::game::clue_type::ClueType;
use crate::game::state::PlayerIndex;
use crate::impl_convention_tech_for_hgroup_clue_tech;

fn is_five_saveable(target: PlayerIndex, pov: &dyn PlayerPOV) -> bool {
    get_chop_index(target, pov)
        .and_then(|chop| pov.card_identity(chop))
        .map(|id| {
            let rank5_mask = pov.static_data().variant.empathy_for_clue(&RANK_5_CLUE).as_bits();
            (1u64 << id) & rank5_mask != 0
        })
        .unwrap_or(false)
}

/// Clue rank 5 to a teammate whose chop card is a 5.
pub struct FiveSave;

const RANK_5_CLUE: Clue = Clue {
    clue_type: ClueType::Rank,
    clue_value: 5,
};

impl ClueTech for FiveSave {
    fn clue_game_actions(&self, player_on_turn_pov: &dyn PlayerPOV) -> Vec<GameAction> {
        let active = player_on_turn_pov.player_on_turn_index();
        let num_players = player_on_turn_pov.static_data().number_of_players as usize;

        (0..num_players)
            .filter(|&p| p != active)
            .filter(|&target| is_five_saveable(target, player_on_turn_pov))
            .map(|target| {
                let touched = touched_cards_for_clue(target, &RANK_5_CLUE, player_on_turn_pov);
                GameAction::Clue {
                    player_index: target,
                    touched_card_deck_indexes: touched,
                    clue: RANK_5_CLUE,
                }
            })
            .collect()
    }

    fn matches_clue(
        &self,
        player_index: PlayerIndex,
        _touched: &[CardDeckIndex],
        clue: &Clue,
        pov: &dyn PlayerPOV,
    ) -> bool {
        if *clue != RANK_5_CLUE {
            return false;
        }
        let giver_pov = giver_pov(pov);
        is_five_saveable(player_index, &giver_pov)
    }

    fn clue_knowledge_updates(
        &self,
        _player_index: PlayerIndex,
        _touched: &[CardDeckIndex],
        _clue: &Clue,
        _pov: &dyn PlayerPOV,
    ) -> Vec<KnowledgeUpdate> {
        vec![]
    }
}

impl HGroupClueTech for FiveSave {}
impl SaveClueTech for FiveSave {}
impl_convention_tech_for_hgroup_clue_tech!(FiveSave, priority::SAVE);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::convention::convention_tech::ConventionTech;
    use crate::engine::knowledge::player_knowledge_state::knowledge_with_visible;
    use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
    use crate::engine::knowledge::team_knowledge::TeamKnowledge;
    use crate::game::card::Empathy;
    use crate::game::clue::Clue;
    use crate::game::clue_type::ClueType;
    use crate::game::deck::unit_test_constants::novariant_constants::{R1_MASK, R5_MASK, Y5_MASK};
    use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
        NOVAR_5_PLAYERS_STATIC_GAME_DATA, initial_five_players_table_state,
    };
    use smallvec::smallvec;

    #[test]
    fn generates_rank5_clue_when_chop_is_a_5() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // chop = R5
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R5_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = FiveSave.game_actions(&pov);

        assert_eq!(
            actions,
            vec![GameAction::Clue {
                player_index: 1,
                touched_card_deck_indexes: smallvec::smallvec![10],
                clue: Clue {
                    clue_type: ClueType::Rank,
                    clue_value: 5
                },
            }]
        );
    }

    #[test]
    fn returns_empty_when_chop_is_not_a_5() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // chop = R1
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(FiveSave.game_actions(&pov).is_empty());
    }

    #[test]
    fn touches_all_5s_in_hand_when_cluing() {
        // Player 1 has two 5s: R5 on chop (oldest) and Y5 (newest).
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10); // oldest = R5 (chop)
        table_state.update_with_draw_action(20); // newest = Y5
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R5_MASK), (20, Y5_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let actions = FiveSave.game_actions(&pov);

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

    #[test]
    fn matches_action_true_when_chop_is_a_5_and_clue_is_rank5() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10);
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R5_MASK)]);
        let mut team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        team_knowledge.player_mut(0).inferred_identities[10] = Some(Empathy::from_bits(R5_MASK).unwrap());
        team_knowledge.player_mut(0).visible_cards |= 1 << 10;
        let pov = LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let action = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: smallvec::smallvec![10],
            clue: Clue {
                clue_type: ClueType::Rank,
                clue_value: 5,
            },
        };
        assert!(FiveSave.matches_action(&action, &pov));
    }

    #[test]
    fn matches_action_false_when_chop_is_not_a_5() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.player_on_turn_index = 1;
        table_state.update_with_draw_action(10);
        table_state.player_on_turn_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov = LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let action = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: smallvec::smallvec![10],
            clue: Clue {
                clue_type: ClueType::Rank,
                clue_value: 5,
            },
        };
        assert!(!FiveSave.matches_action(&action, &pov));
    }
}
