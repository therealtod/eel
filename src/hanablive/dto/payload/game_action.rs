use serde::Deserialize;

use crate::hanablive::TableId;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameActionPayload {
    #[serde(rename = "tableID")]
    pub table_id: TableId,
    pub action: GameActionDto,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameActionListPayload {
    #[serde(rename = "tableID")]
    pub table_id: TableId,
    pub list: Vec<GameActionDto>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum GameActionDto {
    #[serde(rename_all = "camelCase")]
    Draw {
        player_index: usize,
        order: usize,
        suit_index: i32,
        rank: i32,
    },
    #[serde(rename_all = "camelCase")]
    Play {
        player_index: usize,
        order: usize,
        suit_index: i32,
        rank: i32,
    },
    #[serde(rename_all = "camelCase")]
    Discard {
        player_index: usize,
        order: usize,
        suit_index: i32,
        rank: i32,
        failed: bool,
    },
    #[serde(rename_all = "camelCase")]
    Clue {
        clue: Clue,
        giver: usize,
        list: Vec<usize>,
        target: usize,
        turn: usize,
    },
    #[serde(rename_all = "camelCase")]
    Status {
        clues: usize,
        score: usize,
        max_score: usize,
    },
    #[serde(rename_all = "camelCase")]
    Turn {
        num: usize,
        current_player_index: usize,
    },
    #[serde(rename_all = "camelCase")]
    Strike {
        num: usize,
        turn: usize,
        order: usize,
    },
    #[serde(rename_all = "camelCase")]
    GameOver {
        end_condition: u8,
        /// The player responsible for the game ending, or `-1` when not applicable
        /// (e.g. a normal deck-out or a perfect score).
        player_index: i64,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct Clue {
    #[serde(rename = "type")]
    pub clue_type: u8,
    pub value: u8,
}
