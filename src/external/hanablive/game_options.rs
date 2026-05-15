use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GameOptions {
    pub variant: Option<String>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}