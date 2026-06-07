use crate::hanablive::TableId;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardIdentitiesPayload {
    #[serde(rename = "tableID")]
    pub table_id: TableId,
    pub card_identities: Vec<CardIdentity>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardIdentity {
    pub suit_index: usize,
    pub rank: u8,
}
