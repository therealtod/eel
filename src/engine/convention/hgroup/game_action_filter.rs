use crate::engine::convention::hgroup::h_group_core::{
    get_chop_index, is_minimal_clue_value_compliant,
};
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;

type FilterFn = Box<dyn Fn(&GameAction, &dyn PlayerPOV) -> bool + Send + Sync>;

/// A predicate that decides whether a proposed [`GameAction`] is admissible under the
/// conventions. A filter returns `true` to keep the action and `false` to reject it.
///
/// Filters capture their own context (e.g. whether the Early Game restriction is active),
/// so they are constructed fresh at the point of use rather than stored on a tech.
pub struct GameActionFilter {
    filter: FilterFn,
}

impl GameActionFilter {
    pub fn apply(&self, action: &GameAction, pov: &dyn PlayerPOV) -> bool {
        (self.filter)(action, pov)
    }

    /// Rejects clue actions that violate the Minimum Clue Value Principle (MVCP).
    /// Non-clue actions always pass.
    pub fn minimum_clue_value() -> Self {
        GameActionFilter {
            filter: Box::new(|action, pov| {
                if let GameAction::Clue {
                    player_index,
                    touched_card_deck_indexes,
                    clue,
                    ..
                } = action
                {
                    is_minimal_clue_value_compliant(
                        clue,
                        player_index,
                        touched_card_deck_indexes,
                        pov,
                    )
                } else {
                    true
                }
            }),
        }
    }

    /// Early Game restriction on the chop discard.
    ///
    /// While the game is still in the Early Game (`is_early_game`) and the active player has a
    /// productive, good-touch-compliant alternative tech available (`has_productive_alternative`),
    /// the player is not allowed to discard their chop. This rejects a chop discard when both
    /// conditions hold. A *known-trash* chop discard is always allowed: that is a
    /// `DiscardKnownTrash`, not a `DiscardChop`, and never ends the Early Game.
    ///
    /// All non-discard actions, and discards of a non-chop card, always pass.
    pub fn early_game_chop_discard(is_early_game: bool, has_productive_alternative: bool) -> Self {
        let restricted = is_early_game && has_productive_alternative;
        GameActionFilter {
            filter: Box::new(move |action, pov| {
                if !restricted {
                    return true;
                }
                let GameAction::Discard {
                    player_index,
                    card_deck_index,
                    ..
                } = action
                else {
                    return true;
                };
                let is_chop = get_chop_index(*player_index, pov).as_ref() == Some(card_deck_index);
                let is_known_trash = pov.is_known_trash(*card_deck_index);
                !is_chop || is_known_trash
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GameActionFilter;
    use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
    use crate::engine::knowledge::player_knowledge::{knowledge_for_hand, knowledge_with_empathy};
    use crate::engine::knowledge::team_knowledge::TeamKnowledge;
    use crate::game::action::game_action::GameAction;
    use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards::{R1, Y1};
    use crate::game::deck::unit_test_constants::novariant_constants::{R1_MASK, Y1_MASK};
    use crate::game::state::table_state::unit_test_constants::no_variant_constants::{
        NOVAR_5_PLAYERS_STATIC_GAME_DATA, initial_five_players_table_state,
    };

    fn discard(card_deck_index: u8) -> GameAction {
        GameAction::Discard {
            player_index: 0,
            card_deck_index,
            turn: 1,
        }
    }

    #[test]
    fn allows_chop_discard_when_not_restricted() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        for &idx in &[10u8, 20, 30] {
            table_state.update_with_draw_action(idx);
        }
        let knowledge = knowledge_for_hand(&[10, 20, 30]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        // Early game, but no productive alternative → not restricted.
        let filter = GameActionFilter::early_game_chop_discard(true, false);
        assert!(filter.apply(&discard(10), &pov));

        // Productive alternative, but not early game → not restricted.
        let filter = GameActionFilter::early_game_chop_discard(false, true);
        assert!(filter.apply(&discard(10), &pov));
    }

    #[test]
    fn rejects_useful_chop_discard_when_restricted() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        for &idx in &[10u8, 20, 30] {
            table_state.update_with_draw_action(idx);
        }
        let knowledge = knowledge_for_hand(&[10, 20, 30]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let filter = GameActionFilter::early_game_chop_discard(true, true);
        // Card 10 is the chop (oldest, unclued) and is not known trash → rejected.
        assert!(!filter.apply(&discard(10), &pov));
    }

    #[test]
    fn allows_non_chop_discard_when_restricted() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        for &idx in &[10u8, 20, 30] {
            table_state.update_with_draw_action(idx);
        }
        let knowledge = knowledge_for_hand(&[10, 20, 30]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let filter = GameActionFilter::early_game_chop_discard(true, true);
        // Card 30 is not the chop → allowed.
        assert!(filter.apply(&discard(30), &pov));
    }

    #[test]
    fn allows_known_trash_chop_discard_when_restricted() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        // Player 0 draws the chop (42) plus two cards used to seed the R1/Y1 stacks.
        for &idx in &[42u8, 5, 6] {
            table_state.update_with_draw_action(idx);
        }
        table_state.update_with_play_action_of_specific_card(
            5,
            R1.as_variant_card_id(),
            &static_data,
        );
        table_state.update_with_play_action_of_specific_card(
            6,
            Y1.as_variant_card_id(),
            &static_data,
        );

        // Card 42 is the only remaining (oldest, unclued) card → chop. Its empathy is {R1, Y1},
        // both played to the stacks, so it is known trash.
        let knowledge = knowledge_with_empathy(42, R1_MASK | Y1_MASK);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let filter = GameActionFilter::early_game_chop_discard(true, true);
        // A known-trash chop discard is a DiscardKnownTrash, not a DiscardChop → allowed.
        assert!(filter.apply(&discard(42), &pov));
    }

    #[test]
    fn allows_non_discard_actions_when_restricted() {
        let static_data = NOVAR_5_PLAYERS_STATIC_GAME_DATA;
        let mut table_state = initial_five_players_table_state();
        for &idx in &[10u8, 20, 30] {
            table_state.update_with_draw_action(idx);
        }
        let knowledge = knowledge_for_hand(&[10, 20, 30]);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        let pov =
            LightweightPlayerPOV::new(0, &knowledge, &team_knowledge, &table_state, &static_data);

        let filter = GameActionFilter::early_game_chop_discard(true, true);
        let play = GameAction::Play {
            player_index: 0,
            card_deck_index: 10,
            turn: 1,
        };
        assert!(filter.apply(&play, &pov));
    }
}
