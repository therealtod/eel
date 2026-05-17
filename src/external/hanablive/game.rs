use serde::{Deserialize, Serialize};

use super::{Action, Card, Character, GameOptions};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Game {
    pub players: Vec<String>,
    pub deck: Vec<Card>,
    pub actions: Vec<Action>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<GameOptions>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<Vec<Vec<String>>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub characters: Option<Vec<Character>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<String>,
}

impl Game {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
