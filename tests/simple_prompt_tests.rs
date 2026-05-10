mod common;
#[test]
fn all_players_understand_simple_prompt_semantics() {
    let (table_state, static_data, team_knowledge, history, actions) =
        common::load_scenario_with_knowledge(3);
}
