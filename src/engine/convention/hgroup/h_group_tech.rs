use crate::engine::convention::convention_tech::ClueTech;
use crate::engine::convention::hgroup::game_action_filter::GameActionFilter;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;

/// H-Group interpretation priority tiers. Lower value = higher priority.
///
/// `PROMPT` is intentionally equal to `SIMPLE_PLAY_CLUE` so that when both a direct-play
/// and a prompt interpretation match the same clue (e.g. a rank-3 clue touching a slot that
/// could be B3 directly or R3 via a teammate's touched R2), both contribute to the tier-0
/// union. This leaves the receiver with an ambiguous empathy ({B3 ∪ R3}) instead of
/// collapsing to B3, correctly preventing an immediate play until the prompt resolves.
pub mod priority {
    pub const SAVE: u8 = 0;
    pub const SIMPLE_PLAY_CLUE: u8 = 1;
    pub const PROMPT: u8 = 1;
    pub const FINESSE: u8 = 3;
}

/// H-Group refinement of `ClueTech`. Use `impl_convention_tech_for_hgroup_clue_tech!`
/// with an explicit priority constant to wire up `ConventionTech`.
pub trait HGroupClueTech: ClueTech {
    /// Filters applied to this tech's proposed clue actions after generation.
    ///
    /// Default enforces the Minimum Clue Value Principle. Override to return `vec![]`
    /// for techs that intentionally violate MCVP (e.g. future Tempo Clue exceptions).
    fn clue_action_filters(&self) -> Vec<GameActionFilter> {
        vec![GameActionFilter::minimum_clue_value()]
    }

    fn filtered_clue_game_actions(&self, pov: &dyn PlayerPOV) -> Vec<GameAction> {
        let mut actions = self.clue_game_actions(pov);
        let filters = self.clue_action_filters();
        actions.retain(|a| filters.iter().all(|f| f.apply(a, pov)));
        actions
    }
}

/// Save clue techniques.
pub trait SaveClueTech: HGroupClueTech {}

/// Play clue techniques.
pub trait PlayClueTech: HGroupClueTech {}

/// Generates a `ConventionTech` impl for a type that implements `HGroupClueTech`.
///
/// The second argument is the H-Group priority constant (e.g. `priority::SAVE`).
///
/// Usage: `impl_convention_tech_for_hgroup_clue_tech!(MyType, priority::SAVE);`
#[macro_export]
macro_rules! impl_convention_tech_for_hgroup_clue_tech {
    ($t:ty, $priority:expr) => {
        impl $crate::engine::convention::convention_tech::ConventionTech for $t {
            fn name(&self) -> &'static str {
                stringify!($t)
            }
            fn interpretation_priority(&self) -> u8 {
                $priority
            }
            fn game_actions(
                &self,
                pov: &dyn $crate::engine::knowledge::player_pov::PlayerPOV,
            ) -> Vec<$crate::game::action::game_action::GameAction> {
                $crate::engine::convention::hgroup::h_group_tech::HGroupClueTech::filtered_clue_game_actions(self, pov)
            }
            $crate::__impl_clue_tech_matches_and_updates!();
        }
    };
}
