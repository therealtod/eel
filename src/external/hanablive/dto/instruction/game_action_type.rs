use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum GameActionType {
    #[serde(rename = "clue")]
    Clue,
    #[serde(rename = "discard")]
    Discard,
    #[serde(rename = "draw")]
    Draw,
    #[serde(rename = "play")]
    Play,
    #[serde(rename = "status")]
    Status,
    #[serde(rename = "strike")]
    Strike,
    #[serde(rename = "turn")]
    Turn,
}
