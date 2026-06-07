use crate::hanablive::{TableId, UserId};
use serde::Deserialize;

// Example: {"userID":86718,"name":"ilikeeelst","status":0,"tableID":0,"hyphenated":true,"inactive":false}
#[derive(Debug, Clone, Deserialize)]
pub struct User {
    #[serde(rename = "userID")]
    pub user_id: UserId,
    pub name: String,
    pub status: u8,
    #[serde(rename = "tableID")]
    pub table_id: TableId,
    pub hyphenated: bool,
    pub inactive: bool,
}
