use async_trait::async_trait;

use super::Handler;
use crate::external::hanablive::bot::HanabLiveBot;
use crate::external::hanablive::dto::instruction_type::InstructionType;
use crate::external::hanablive::handler::HandlerError;

pub struct NoOpHandler;

#[async_trait]
impl Handler for NoOpHandler {
    fn supports(&self, _instruction_type: InstructionType) -> bool {
        true
    }

    async fn handle(&self, _payload: &str, _bot: &mut HanabLiveBot) -> Result<(), HandlerError> {
        Ok(())
    }

    fn next(&self) -> Option<&dyn Handler> {
        None
    }
}
