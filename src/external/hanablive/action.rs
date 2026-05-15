use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    Play,
    Discard,
    ColorClue,
    RankClue,
    EndGame,
}

impl ActionType {
    fn as_u8(self) -> u8 {
        match self {
            ActionType::Play => 0,
            ActionType::Discard => 1,
            ActionType::ColorClue => 2,
            ActionType::RankClue => 3,
            ActionType::EndGame => 4,
        }
    }
}

impl<'de> serde::Deserialize<'de> for ActionType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        match value {
            0 => Ok(ActionType::Play),
            1 => Ok(ActionType::Discard),
            2 => Ok(ActionType::ColorClue),
            3 => Ok(ActionType::RankClue),
            4 => Ok(ActionType::EndGame),
            _ => Err(serde::de::Error::custom(format!(
                "Invalid action type: {}",
                value
            ))),
        }
    }
}

impl Serialize for ActionType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(self.as_u8())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    #[serde(rename = "type")]
    pub action_type: ActionType,
    pub target: usize,
    pub value: Option<usize>,
}