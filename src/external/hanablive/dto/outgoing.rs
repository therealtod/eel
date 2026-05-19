use serde::Serialize;

pub trait Instruction {
    fn label(&self) -> &str;
    fn payload(&self) -> String;

    fn to_websocket_message(&self) -> String {
        format!("{} {}", self.label(), self.payload())
    }
}

#[derive(Debug, Clone, Serialize)]
struct TableJoinPayload {
    #[serde(rename = "tableID")]
    table_id: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>,
}

pub struct TableJoin {
    pub table_id: usize,
}

impl Instruction for TableJoin {
    fn label(&self) -> &str {
        "tableJoin"
    }

    fn payload(&self) -> String {
        serde_json::to_string(&TableJoinPayload {
            table_id: self.table_id,
            password: None,
        })
        .unwrap()
    }
}

pub struct PasswordProtectedTableJoin {
    pub table_id: usize,
    pub password: String,
}

impl Instruction for PasswordProtectedTableJoin {
    fn label(&self) -> &str {
        "tableJoin"
    }

    fn payload(&self) -> String {
        serde_json::to_string(&TableJoinPayload {
            table_id: self.table_id,
            password: Some(self.password.clone()),
        })
        .unwrap()
    }
}

#[derive(Debug, Clone, Serialize)]
struct GetGameInfoPayload {
    #[serde(rename = "tableID")]
    table_id: usize,
}

pub struct GetGameInfo1 {
    pub table_id: usize,
}

impl Instruction for GetGameInfo1 {
    fn label(&self) -> &str {
        "getGameInfo1"
    }

    fn payload(&self) -> String {
        serde_json::to_string(&GetGameInfoPayload {
            table_id: self.table_id,
        })
        .unwrap()
    }
}

pub struct GetGameInfo2 {
    pub table_id: usize,
}

impl Instruction for GetGameInfo2 {
    fn label(&self) -> &str {
        "getGameInfo2"
    }

    fn payload(&self) -> String {
        serde_json::to_string(&GetGameInfoPayload {
            table_id: self.table_id,
        })
        .unwrap()
    }
}
