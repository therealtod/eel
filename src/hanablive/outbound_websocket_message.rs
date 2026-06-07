use crate::hanablive::TableId;
use crate::hanablive::action::ActionType;
use crate::hanablive::dto::payload::table_join::TableJoin;

#[derive(Debug)]
pub enum OutboundWebsocketMessage {
    GameAction {
        table_id: TableId,
        action_type: ActionType,
        target: usize,
        value: Option<usize>,
    },
    GetGameInfo1(TableId),
    GetGameInfo2(TableId),
    Loaded(TableId),
    TableJoin(TableId, Option<String>),
    TableLeave(TableId),
    TableUnattend(TableId),
    TableSpectate,
}

impl OutboundWebsocketMessage {
    pub fn serialize(&self) -> String {
        match self {
            Self::GameAction {
                table_id,
                action_type,
                target,
                value,
            } => {
                let type_val = action_type.as_u8();
                match value {
                    Some(v) => format!(
                        "action {{\"tableID\":{table_id},\"type\":{type_val},\"target\":{target},\"value\":{v}}}"
                    ),
                    None => format!(
                        "action {{\"tableID\":{table_id},\"type\":{type_val},\"target\":{target}}}"
                    ),
                }
            }
            Self::GetGameInfo1(id) => format!("getGameInfo1 {{\"tableID\":{id}}}"),
            Self::GetGameInfo2(id) => format!("getGameInfo2 {{\"tableID\":{id}}}"),
            Self::Loaded(id) => format!("loaded {{\"tableID\":{id}}}"),
            Self::TableJoin(table_id, password) => {
                let join = TableJoin {
                    table_id: *table_id,
                    password: password.clone(),
                };
                format!("tableJoin {}", serde_json::to_string(&join).unwrap())
            }
            Self::TableLeave(id) => format!("tableLeave {{\"tableID\":{id}}}"),
            Self::TableUnattend(id) => format!("tableUnattend {{\"tableID\":{id}}}"),
            Self::TableSpectate => "tableSpectate {}".to_string(),
        }
    }
}
