use crate::external::hanablive::bot::BotEvent;
use crate::external::hanablive::bot::state::common::CommonState;
use crate::external::hanablive::bot::state::table_joined::TableJoinedState;
use crate::external::hanablive::dto::outgoing::{PasswordProtectedTableJoin, TableJoin};

#[derive(Debug)]
pub struct LoggedInState {
    pub common_state: CommonState,
    pub sender: tokio::sync::mpsc::UnboundedSender<BotEvent>,
}

impl LoggedInState {
    pub async fn join_table(&self, table_id: usize) -> Result<(), String> {
        self.sender
            .send(BotEvent::SendInstruction(Box::new(TableJoin { table_id })))
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn join_table_with_password(
        &self,
        table_id: usize,
        password: String,
    ) -> Result<(), String> {
        self.sender
            .send(BotEvent::SendInstruction(Box::new(
                PasswordProtectedTableJoin { table_id, password },
            )))
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn transition_to_table_joined(self, table_id: usize) -> TableJoinedState {
        TableJoinedState {
            common_state: self.common_state,
            table_id,
            sender: self.sender,
        }
    }
}
