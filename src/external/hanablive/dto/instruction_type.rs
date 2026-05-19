use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum InstructionType {
    #[serde(rename = "chat")]
    Chat,
    #[serde(rename = "chatList")]
    ChatList,
    #[serde(rename = "chatTyping")]
    ChatTyping,
    #[serde(rename = "gameAction")]
    GameAction,
    #[serde(rename = "gameActionList")]
    GameActionList,
    #[serde(rename = "gameHistory")]
    GameHistory,
    #[serde(rename = "joined")]
    Joined,
    #[serde(rename = "hypoStart")]
    HypoStart,
    #[serde(rename = "hypoEnd")]
    HypoEnd,
    #[serde(rename = "init")]
    Init,
    #[serde(rename = "left")]
    Left,
    #[serde(rename = "replayIndicator")]
    ReplayIndicator,
    #[serde(rename = "replaySegment")]
    ReplaySegment,
    #[serde(rename = "table")]
    Table,
    #[serde(rename = "tableGone")]
    TableGone,
    #[serde(rename = "tableList")]
    TableList,
    #[serde(rename = "tableStart")]
    TableStart,
    #[serde(rename = "tableProgress")]
    TableProgress,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "userLeft")]
    UserLeft,
    #[serde(rename = "userList")]
    UserList,
    #[serde(rename = "userInactive")]
    UserInactive,
    #[serde(rename = "userGone")]
    UserGone,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "welcome")]
    Welcome,
}

impl InstructionType {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "chat" => Some(InstructionType::Chat),
            "chatList" => Some(InstructionType::ChatList),
            "chatTyping" => Some(InstructionType::ChatTyping),
            "gameAction" => Some(InstructionType::GameAction),
            "gameActionList" => Some(InstructionType::GameActionList),
            "gameHistory" => Some(InstructionType::GameHistory),
            "joined" => Some(InstructionType::Joined),
            "hypoStart" => Some(InstructionType::HypoStart),
            "hypoEnd" => Some(InstructionType::HypoEnd),
            "init" => Some(InstructionType::Init),
            "left" => Some(InstructionType::Left),
            "replayIndicator" => Some(InstructionType::ReplayIndicator),
            "replaySegment" => Some(InstructionType::ReplaySegment),
            "table" => Some(InstructionType::Table),
            "tableGone" => Some(InstructionType::TableGone),
            "tableList" => Some(InstructionType::TableList),
            "tableStart" => Some(InstructionType::TableStart),
            "tableProgress" => Some(InstructionType::TableProgress),
            "user" => Some(InstructionType::User),
            "userLeft" => Some(InstructionType::UserLeft),
            "userList" => Some(InstructionType::UserList),
            "userInactive" => Some(InstructionType::UserInactive),
            "userGone" => Some(InstructionType::UserGone),
            "warning" => Some(InstructionType::Warning),
            "welcome" => Some(InstructionType::Welcome),
            _ => None,
        }
    }
}
