use serde::Deserialize;

use super::game_action_data::GameActionData;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameActionListData {
    #[serde(rename = "tableID")]
    pub table_id: usize,
    pub list: Vec<GameActionData>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameAction {
    #[serde(rename = "tableID")]
    pub table_id: usize,
    pub action: GameActionData,
}
