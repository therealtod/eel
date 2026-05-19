use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Spectator {
    pub name: String,
    pub shadowing_player_index: usize,
    pub shadowing_player_username: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Table {
    pub id: usize,
    pub name: String,
    pub password_protected: bool,
    pub joined: bool,
    pub num_players: usize,
    pub starting_player: usize,
    pub owned: bool,
    pub running: bool,
    pub variant: String,
    pub options: super::game_options::GameOptions,
    pub timed: bool,
    pub time_base: usize,
    pub time_per_turn: usize,
    pub shared_replay: bool,
    pub progress: usize,
    pub players: Vec<String>,
    pub spectators: Vec<Spectator>,
    pub max_players: usize,
}
