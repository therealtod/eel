use serde::Deserialize;
use serde::de::DeserializeOwned;
use tracing::warn;

use crate::hanablive::dto::payload::card_identity::CardIdentitiesPayload;
use crate::hanablive::dto::payload::chat::Chat;
use crate::hanablive::dto::payload::chat_list::ChatList;
use crate::hanablive::dto::payload::game_action::{GameActionListPayload, GameActionPayload};
use crate::hanablive::dto::payload::game_init_data::GameInitData;
use crate::hanablive::dto::payload::table::Table;
use crate::hanablive::dto::payload::table_start::TableStart;
use crate::hanablive::dto::payload::user::User;
use crate::hanablive::dto::payload::welcome::Welcome;

#[derive(Clone, Debug, Deserialize)]
pub enum InboundWebsocketMessage {
    #[serde(rename = "cardIdentities")]
    CardIdentities(CardIdentitiesPayload),
    #[serde(rename = "chat")]
    Chat(Chat),
    #[serde(rename = "chatList")]
    ChatList(ChatList),
    #[serde(rename = "chatTyping")]
    ChatTyping,
    #[serde(rename = "gameAction")]
    GameAction(GameActionPayload),
    #[serde(rename = "gameActionList")]
    GameActionList(GameActionListPayload),
    #[serde(rename = "gameHistory")]
    GameHistory,
    #[serde(rename = "hypoEnd")]
    HypoEnd,
    #[serde(rename = "hypoStart")]
    HypoStart,
    #[serde(rename = "init")]
    Init(GameInitData),
    #[serde(rename = "joined")]
    Joined,
    #[serde(rename = "left")]
    Left,
    #[serde(rename = "replayIndicator")]
    ReplayIndicator,
    #[serde(rename = "replaySegment")]
    ReplaySegment,
    #[serde(rename = "table")]
    Table(Table),
    #[serde(rename = "tableGone")]
    TableGone,
    #[serde(rename = "tableList")]
    TableList(Vec<Table>),
    #[serde(rename = "tableProgress")]
    TableProgress,
    #[serde(rename = "tableStart")]
    TableStart(TableStart),
    #[serde(rename = "user")]
    User(User),
    #[serde(rename = "userGone")]
    UserGone,
    #[serde(rename = "userInactive")]
    UserInactive,
    #[serde(rename = "userLeft")]
    UserLeft,
    #[serde(rename = "userList")]
    UserList(Vec<User>),
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "welcome")]
    Welcome(Welcome),
    Unknown,
}

impl InboundWebsocketMessage {
    pub fn label(&self) -> &str {
        match self {
            InboundWebsocketMessage::CardIdentities(_) => "cardIdentities",
            InboundWebsocketMessage::Chat(_) => "chat",
            InboundWebsocketMessage::ChatList(_) => "chatList",
            InboundWebsocketMessage::ChatTyping => "chatTyping",
            InboundWebsocketMessage::GameAction(_) => "gameAction",
            InboundWebsocketMessage::GameActionList(_) => "gameActionList",
            InboundWebsocketMessage::GameHistory => "gameHistory",
            InboundWebsocketMessage::HypoEnd => "hypoEnd",
            InboundWebsocketMessage::HypoStart => "hypoStart",
            InboundWebsocketMessage::Init(_) => "init",
            InboundWebsocketMessage::Joined => "joined",
            InboundWebsocketMessage::Left => "left",
            InboundWebsocketMessage::ReplayIndicator => "replayIndicator",
            InboundWebsocketMessage::ReplaySegment => "replaySegment",
            InboundWebsocketMessage::Table(_) => "table",
            InboundWebsocketMessage::TableGone => "tableGone",
            InboundWebsocketMessage::TableList(_) => "tableList",
            InboundWebsocketMessage::TableProgress => "tableProgress",
            InboundWebsocketMessage::TableStart(_) => "tableStart",
            InboundWebsocketMessage::User(_) => "user",
            InboundWebsocketMessage::UserGone => "userGone",
            InboundWebsocketMessage::UserInactive => "userInactive",
            InboundWebsocketMessage::UserLeft => "userLeft",
            InboundWebsocketMessage::UserList(_) => "userList",
            InboundWebsocketMessage::Warning => "warning",
            InboundWebsocketMessage::Welcome(_) => "welcome",
            InboundWebsocketMessage::Unknown => "unknown",
        }
    }
    fn parse_json<T: DeserializeOwned>(payload: &str, f: fn(T) -> Self) -> Self {
        match serde_json::from_str(payload) {
            Ok(value) => f(value),
            Err(e) => {
                warn!(payload = ?payload, "Failed to parse websocket message payload: {}", e);
                warn!("Failed to parse websocket message payload: {}", e);
                Self::Unknown
            }
        }
    }

    #[must_use]
    pub fn parse(raw: &str) -> Self {
        let (label, payload) = raw.split_once(' ').unwrap_or((raw, ""));
        match label {
            "cardIdentities" => Self::parse_json(payload, Self::CardIdentities),
            "chat" => Self::parse_json(payload, Self::Chat),
            "chatList" => Self::parse_json(payload, Self::ChatList),
            "chatTyping" => Self::ChatTyping,
            "gameAction" => Self::parse_json(payload, Self::GameAction),
            "gameActionList" => Self::parse_json(payload, Self::GameActionList),
            "gameHistory" => Self::GameHistory,
            "hypoEnd" => Self::HypoEnd,
            "hypoStart" => Self::HypoStart,
            "init" => Self::parse_json(payload, Self::Init),
            "joined" => Self::Joined,
            "left" => Self::Left,
            "replayIndicator" => Self::ReplayIndicator,
            "replaySegment" => Self::ReplaySegment,
            "table" => Self::parse_json(payload, Self::Table),
            "tableGone" => Self::TableGone,
            "tableList" => Self::parse_json(payload, Self::TableList),
            "tableProgress" => Self::TableProgress,
            "tableStart" => Self::parse_json(payload, Self::TableStart),
            "user" => Self::parse_json(payload, Self::User),
            "userGone" => Self::UserGone,
            "userInactive" => Self::UserInactive,
            "userLeft" => Self::UserLeft,
            "userList" => Self::parse_json(payload, Self::UserList),
            "warning" => Self::Warning,
            "welcome" => Self::parse_json(payload, Self::Welcome),
            _ => {
                warn!("Unknown websocket message label: {label}");
                Self::Unknown
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_game_action_list_with_camel_case_variant_fields_and_negative_values() {
        let raw = r#"gameActionList {"tableID":2052,"list":[{"type":"draw","playerIndex":0,"order":0,"suitIndex":2,"rank":1},{"type":"draw","playerIndex":0,"order":1,"suitIndex":1,"rank":4},{"type":"draw","playerIndex":0,"order":2,"suitIndex":2,"rank":1},{"type":"draw","playerIndex":0,"order":3,"suitIndex":0,"rank":4},{"type":"draw","playerIndex":0,"order":4,"suitIndex":2,"rank":4},{"type":"draw","playerIndex":1,"order":5,"suitIndex":-1,"rank":-1},{"type":"draw","playerIndex":1,"order":6,"suitIndex":-1,"rank":-1},{"type":"draw","playerIndex":1,"order":7,"suitIndex":-1,"rank":-1},{"type":"draw","playerIndex":1,"order":8,"suitIndex":-1,"rank":-1},{"type":"draw","playerIndex":1,"order":9,"suitIndex":-1,"rank":-1}]}"#;
        let msg = InboundWebsocketMessage::parse(raw);
        assert!(
            !matches!(msg, InboundWebsocketMessage::Unknown),
            "Failed to parse gameActionList containing camelCase variant fields and negative values"
        );
        if let InboundWebsocketMessage::GameActionList(payload) = msg {
            assert_eq!(payload.table_id, 2052);
            assert_eq!(payload.list.len(), 10);
        } else {
            panic!("Expected GameActionList variant");
        }
    }
}
