use crate::hanablive::dto::payload::chat::Chat;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ChatList {
    pub list: Vec<Chat>,
}
