use async_trait::async_trait;
use serde_json::Value;

use super::Handler;
use crate::external::hanablive::bot::HanabLiveBot;
use crate::external::hanablive::dto::instruction_type::InstructionType;
use crate::external::hanablive::dto::outgoing::GetGameInfo1;
use crate::external::hanablive::handler::{ChatMessageHandler, HandlerError};

pub struct TableStartHandler;

#[async_trait]
impl Handler for TableStartHandler {
    fn supports(&self, instruction_type: InstructionType) -> bool {
        matches!(instruction_type, InstructionType::TableStart)
    }

    async fn handle(&self, payload: &str, bot: &mut HanabLiveBot) -> Result<(), HandlerError> {
        let tree: Value = serde_json::from_str(payload)?;
        let table_id = tree["tableID"].as_u64().unwrap_or(0) as usize;
        bot.send_instruction(GetGameInfo1 { table_id }).await;
        Ok(())
    }

    fn next(&self) -> Option<&dyn Handler> {
        Some(&ChatMessageHandler)
    }
}
