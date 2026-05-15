use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    Play,
    Discard,
    ColorClue,
    RankClue,
    EndGame,
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    #[serde(rename = "type")]
    pub action_type: ActionType,
    pub target: usize,
    pub value: Option<usize>,
}