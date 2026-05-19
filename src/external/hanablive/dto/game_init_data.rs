use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::game_options::GameOptions;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameInitData {
    #[serde(rename = "tableID")]
    pub table_id: usize,
    pub player_names: Vec<String>,
    pub our_player_index: usize,
    pub spectating: bool,
    pub shadowing: bool,
    pub replay: bool,
    #[serde(rename = "databaseID")]
    pub database_id: i32,
    pub has_custom_seed: bool,
    pub seed: String,
    pub datetime_started: DateTime<Utc>,
    pub datetime_finished: DateTime<Utc>,
    pub options: GameOptions,
    pub character_assignments: Vec<serde_json::Value>,
    pub character_metadata: Vec<serde_json::Value>,
    pub shared_replay: bool,
    pub shared_replay_leader: String,
    pub shared_replay_segment: usize,
    pub shared_replay_eff_mod: usize,
    pub paused: bool,
    pub pause_player_index: i32,
    pub pause_queued: bool,
}
