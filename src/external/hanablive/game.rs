use serde::Deserialize;

use super::{Action, Card, Character, GameOptions};

#[derive(Debug, Clone, Deserialize)]
pub struct Game {
    pub players: Vec<String>,
    pub deck: Vec<Card>,
    pub actions: Vec<Action>,
    #[serde(default)]
    pub options: Option<GameOptions>,
    #[serde(default)]
    pub notes: Option<Vec<Vec<String>>>,
    #[serde(default)]
    pub characters: Option<Vec<Character>>,
    pub id: Option<i64>,
    pub seed: Option<String>,
}

impl Game {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}