use crate::engine::convention::convention_set::ConventionSet;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;

pub trait ActionSelectionStrategy {
    /// Select the best action for the active player.
    ///
    /// The player's state is available through `player_pov`, and the agreed-upon
    /// conventions through `convention_set`.
    fn select_active_player_action(
        &self,
        player_pov: &dyn PlayerPOV,
        convention_set: &dyn ConventionSet,
    ) -> GameAction;
}
