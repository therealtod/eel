use super::get_clue_focus;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::card::CardDeckIndex;
use crate::game::clue::Clue;
use crate::game::{MAX_CLUE_VALUES_PER_TYPE, MAX_HAND_SIZE};
use smallvec::SmallVec;

/// Returns all valid clues for `target_player_index` paired with their focus card index.
///
/// For every (`clue_type`, `clue_value`) combination that touches at least one fully-known card in
/// the target's hand and produces a defined focus, one `(GameAction::Clue, focus_index)` entry
/// is emitted. Callers filter the list on whatever focus-card condition they need.
#[allow(clippy::cast_possible_truncation)]
pub fn clues_for_player_with_focus(
    target_player_index: usize,
    player_pov: &dyn PlayerPOV,
) -> Vec<(GameAction, CardDeckIndex)> {
    let table_state = player_pov.table_state();
    let static_data = player_pov.static_data();

    if table_state.clue_token_bank.whole_clue_tokens_count() == 0 {
        return Vec::new();
    }

    let hand_cards: Vec<_> = table_state.hands[target_player_index].cards().to_vec();
    let mut result = Vec::new();

    for clue_type in &static_data.variant.clue_types {
        for clue_value in 0..MAX_CLUE_VALUES_PER_TYPE {
            let empathy = static_data.variant.empathy_by_clue(*clue_type, clue_value);
            let empathy_bits = empathy.as_bits();

            let touched: SmallVec<[CardDeckIndex; MAX_HAND_SIZE]> = hand_cards
                .iter()
                .copied()
                .filter(|&idx| {
                    player_pov
                        .card_identity(idx)
                        .is_some_and(|id| (1u64 << id) & empathy_bits != 0)
                })
                .collect();

            if touched.is_empty() {
                continue;
            }

            let clue = Clue {
                clue_type: *clue_type,
                clue_value: clue_value as u8,
            };

            if let Some(focus_idx) = get_clue_focus(target_player_index, &touched, player_pov) {
                result.push((
                    GameAction::Clue {
                        player_index: target_player_index,
                        touched_card_deck_indexes: touched,
                        clue,
                        turn: player_pov.table_state().current_turn,
                    },
                    focus_idx,
                ));
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
    use crate::engine::knowledge::player_knowledge::knowledge_with_visible;
    use crate::engine::knowledge::team_knowledge::TeamKnowledge;
    use crate::game::clue_type::ClueType;
    use crate::game::deck::unit_test_constants::novariant_constants::{R1_MASK, R2_MASK};
    use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
        NOVAR_5_PLAYERS_STATIC_GAME_DATA, initial_five_players_table_state,
    };

    // ── clues_for_player_with_focus ────────────────────────────────────────

    #[test]
    fn clues_for_player_returns_empty_when_no_card_identity_is_known() {
        // Player 0 does not know card 10's identity → no clue can be computed.
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[]); // card 10 not visible
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        assert!(clues_for_player_with_focus(1, &pov).is_empty());
    }

    #[test]
    fn clues_for_player_returns_one_entry_per_clue_that_touches_known_card() {
        // Player 1 has R1 (card 10). Color-red and rank-1 clues both touch it.
        // Both produce focus = 10 (the only card = chop).
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10);
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R1_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let result = clues_for_player_with_focus(1, &pov);

        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|(_, focus)| *focus == 10));
        assert!(result.iter().any(|(action, _)| matches!(action,
            GameAction::Clue { clue, .. } if clue.clue_type == ClueType::Color // color clue
        )));
        assert!(result.iter().any(|(action, _)| matches!(action,
            GameAction::Clue { clue, .. } if clue.clue_type == ClueType::Rank // rank clue
        )));
    }

    #[test]
    fn clues_for_player_focus_is_chop_when_chop_is_touched() {
        // Player 1 hand oldest→newest: [card 10 = R2 (chop), card 20 = R1].
        // Red clue touches both → focus = chop (10, R2).
        // Rank-1 clue touches only card 20 → focus = 20 (R1).
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        table_state.active_player_index = 1;
        table_state.update_with_draw_action(10); // R2, oldest
        table_state.update_with_draw_action(20); // R1, newest
        table_state.active_player_index = 0;

        let knowledge = knowledge_with_visible(0, &[(10, R2_MASK), (20, R1_MASK)]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let result = clues_for_player_with_focus(1, &pov);

        let red_entry = result.iter().find(|(action, _)| {
            matches!(action,
                GameAction::Clue { clue, touched_card_deck_indexes, .. }
                if clue.clue_type == ClueType::Color && touched_card_deck_indexes.len() == 2
            )
        });
        assert_eq!(
            red_entry.map(|(_, f)| *f),
            Some(10),
            "red clue focus should be chop (R2)"
        );

        let rank1_entry = result.iter().find(|(action, _)| {
            matches!(action,
                GameAction::Clue { clue, touched_card_deck_indexes, .. }
                if clue.clue_type == ClueType::Rank && *touched_card_deck_indexes == [20u8].into()
            )
        });
        assert_eq!(
            rank1_entry.map(|(_, f)| *f),
            Some(20),
            "rank-1 clue focus should be R1"
        );
    }
}
