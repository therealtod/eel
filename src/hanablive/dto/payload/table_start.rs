use crate::hanablive::TableId;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TableStart {
    #[serde(rename = "tableID")]
    pub table_id: TableId,
}
