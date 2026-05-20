use crate::engine::convention::hgroup::h_group_core::is_minimal_clue_value_compliant;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;

pub struct GameActionFilter {
    filter: fn(&GameAction, &dyn PlayerPOV) -> bool,
}

impl GameActionFilter {
    pub fn apply(&self, action: &GameAction, pov: &dyn PlayerPOV) -> bool {
        (self.filter)(action, pov)
    }

    /// Rejects clue actions that violate the Minimum Clue Value Principle (MVCP).
    /// Non-clue actions always pass.
    pub fn minimum_clue_value() -> Self {
        GameActionFilter {
            filter: |action, pov| {
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
            },
        }
    }
}
