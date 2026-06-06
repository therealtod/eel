use crate::common;
use eel::engine::action_selection_strategy::ActionSelectionStrategy;
use eel::engine::convention::hgroup::h_group_convention_set::{HGroupConventionSet, HGroupLevel};
use eel::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
use eel::engine::tree_action_selection_strategy::TreeActionSelectionStrategy;
use eel::game::action::game_action::GameAction;

/// Load a named search regression scenario and ask the engine for its best action.
pub fn search_best_action(name: &str) -> GameAction {
    let (table_state, static_data, team_knowledge, _, _) =
        common::load_scenario_by_name_with_knowledge(name);
    let active = table_state.active_player_index;
    let knowledge = team_knowledge.player(active).clone();
    let pov = LightweightPlayerPOV::new(
        active,
        &knowledge,
        &team_knowledge,
        &table_state,
        &static_data,
    );
    let strategy = TreeActionSelectionStrategy::default();
    let conventions = HGroupConventionSet::for_level(HGroupLevel::One, &static_data.variant);
    strategy.select_active_player_action(&pov, &conventions)
}
