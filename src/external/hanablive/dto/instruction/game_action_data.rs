use serde::Deserialize;

use super::game_action_type::GameActionType;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum GameActionData {
    Draw(DrawActionData),
    Play(PlayActionData),
    Discard(DiscardActionData),
    Clue(ClueActionData),
    Turn(TurnActionData),
    Status(StatusActionData),
    Strike(StrikeActionData),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrawActionData {
    pub player_index: usize,
    pub order: usize,
    pub suit_index: usize,
    pub rank: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayActionData {
    pub player_index: usize,
    pub order: usize,
    pub suit_index: usize,
    pub rank: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscardActionData {
    pub player_index: usize,
    pub order: usize,
    pub suit_index: usize,
    pub rank: usize,
    pub failed: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClueActionData {
    pub clue: ClueValue,
    pub giver: usize,
    pub list: Vec<usize>,
    pub target: usize,
    pub turn: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClueValue {
    #[serde(rename = "type")]
    pub clue_type: usize,
    pub value: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnActionData {
    pub num: usize,
    pub current_player_index: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusActionData {
    pub clues: usize,
    pub score: usize,
    pub max_score: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrikeActionData {
    pub num: usize,
    pub turn: usize,
    pub order: usize,
}

impl GameActionData {
    pub fn action_type(&self) -> GameActionType {
        match self {
            GameActionData::Draw(_) => GameActionType::Draw,
            GameActionData::Play(_) => GameActionType::Play,
            GameActionData::Discard(_) => GameActionType::Discard,
            GameActionData::Clue(_) => GameActionType::Clue,
            GameActionData::Turn(_) => GameActionType::Turn,
            GameActionData::Status(_) => GameActionType::Status,
            GameActionData::Strike(_) => GameActionType::Strike,
        }
    }
}
