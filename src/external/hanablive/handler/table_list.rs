use async_trait::async_trait;

use super::Handler;
use crate::external::hanablive::bot::HanabLiveBot;
use crate::external::hanablive::dto::instruction_type::InstructionType;
use crate::external::hanablive::dto::table::Table;
use crate::external::hanablive::handler::{HandlerError, TableHandler};

pub struct TableListHandler;

#[async_trait]
impl Handler for TableListHandler {
    fn supports(&self, instruction_type: InstructionType) -> bool {
        matches!(instruction_type, InstructionType::TableList)
    }

    async fn handle(&self, payload: &str, bot: &mut HanabLiveBot) -> Result<(), HandlerError> {
        let tables: Vec<Table> = serde_json::from_str(payload)?;
        bot.set_tables(tables).await;
        Ok(())
    }

    fn next(&self) -> Option<&dyn Handler> {
        Some(&TableHandler)
    }
}
