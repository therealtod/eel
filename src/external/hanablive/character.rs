use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Character {
    pub name: String,
    pub metadata: i32,
}