use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub msg: String,
    pub who: String,
    pub discord: bool,
    pub server: bool,
    pub datetime: DateTime<Utc>,
    pub room: String,
    pub recipient: String,
}
