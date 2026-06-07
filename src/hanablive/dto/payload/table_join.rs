use crate::hanablive::TableId;

#[derive(Debug, serde::Serialize)]
pub struct TableJoin {
    #[serde(rename = "tableID")]
    pub table_id: TableId,
    pub password: Option<String>,
}
