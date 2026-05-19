use async_trait::async_trait;

use super::Handler;
use crate::external::hanablive::bot::HanabLiveBot;
use crate::external::hanablive::dto::instruction_type::InstructionType;
use crate::external::hanablive::dto::table::Table;
use crate::external::hanablive::handler::{HandlerError, InitHandler};

pub struct TableHandler;

#[async_trait]
impl Handler for TableHandler {
    fn supports(&self, instruction_type: InstructionType) -> bool {
        matches!(instruction_type, InstructionType::Table)
    }

    async fn handle(&self, payload: &str, bot: &mut HanabLiveBot) -> Result<(), HandlerError> {
        let table: Table = serde_json::from_str(payload)?;
        bot.put_table(table).await;
        Ok(())
    }

    fn next(&self) -> Option<&dyn Handler> {
        Some(&InitHandler)
    }
}
