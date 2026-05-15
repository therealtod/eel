use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Character {
    pub name: String,
    pub metadata: i32,
}