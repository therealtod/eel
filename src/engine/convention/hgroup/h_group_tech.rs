use crate::engine::convention::convention_tech::ClueTech;
use crate::engine::convention::hgroup::game_action_filter::GameActionFilter;

/// H-Group interpretation priority tiers. Lower value = higher priority.
pub mod priority {
    pub const SAVE: u8 = 0;
    pub const SIMPLE_PLAY_CLUE: u8 = 1;
    pub const PROMPT: u8 = 2;
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
            fn name(&self) -> &'static str { stringify!($t) }
            fn interpretation_priority(&self) -> u8 {
                $priority
            }
            fn game_actions(&self, pov: &dyn $crate::engine::knowledge::player_pov::PlayerPOV) -> Vec<$crate::game::action::game_action::GameAction> {
                let mut actions = $crate::engine::convention::convention_tech::ClueTech::clue_game_actions(self, pov);
                let filters = $crate::engine::convention::hgroup::h_group_tech::HGroupClueTech::clue_action_filters(self);
                actions.retain(|a| filters.iter().all(|f| f.apply(a, pov)));
                actions
            }
            fn matches_action(&self, action: &$crate::game::action::game_action::GameAction, pov: &dyn $crate::engine::knowledge::player_pov::PlayerPOV) -> bool {
                if let GameAction::Clue { player_index, touched_card_deck_indexes, clue } = action {
                        $crate::engine::convention::convention_tech::ClueTech::matches_clue(self, *player_index, touched_card_deck_indexes, clue, pov)
                } else {
                    false
                }
            }
            fn knowledge_updates(&self, action: &$crate::game::action::game_action::GameAction, pov: &dyn $crate::engine::knowledge::player_pov::PlayerPOV) -> Vec<$crate::engine::knowledge::knowledge_update::KnowledgeUpdate> {
                if let GameAction::Clue { player_index, touched_card_deck_indexes, clue } = action {
                    $crate::engine::convention::convention_tech::ClueTech::clue_knowledge_updates(self, *player_index, touched_card_deck_indexes, clue, pov)
                } else { vec![] }
            }
        }
    };
}
