use async_trait::async_trait;

use super::Handler;
use crate::external::hanablive::bot::HanabLiveBot;
use crate::external::hanablive::dto::instruction_type::InstructionType;
use crate::external::hanablive::handler::{HandlerError, TableListHandler};

pub struct WelcomeHandler;

#[async_trait]
impl Handler for WelcomeHandler {
    fn supports(&self, instruction_type: InstructionType) -> bool {
        matches!(instruction_type, InstructionType::Welcome)
    }

    async fn handle(&self, _payload: &str, _bot: &mut HanabLiveBot) -> Result<(), HandlerError> {
        Ok(())
    }

    fn next(&self) -> Option<&dyn Handler> {
        Some(&TableListHandler)
    }
}
