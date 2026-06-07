use crate::hanablive::action::Action;
use crate::hanablive::dto::card::Card;
use crate::hanablive::dto::character::Character;
use crate::hanablive::game_options::GameOptions;
use serde::{Deserialize, Serialize};

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
    /// Deserialize a [`Game`] from a JSON string.
    ///
    /// # Errors
    ///
    /// Returns [`serde_json::Error`] if the input is not valid JSON or does not match the
    /// expected schema.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize this [`Game`] to a compact JSON string.
    ///
    /// # Errors
    ///
    /// Returns [`serde_json::Error`] if serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Serialize this [`Game`] to a pretty-printed JSON string.
    ///
    /// # Errors
    ///
    /// Returns [`serde_json::Error`] if serialization fails.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
