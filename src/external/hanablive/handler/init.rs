use async_trait::async_trait;

use super::Handler;
use crate::external::hanablive::bot::HanabLiveBot;
use crate::external::hanablive::dto::instruction_type::InstructionType;
use crate::external::hanablive::dto::game_init_data::GameInitData;
use crate::external::hanablive::handler::{ChatMessageHandler, HandlerError};

pub struct InitHandler;

#[async_trait]
impl Handler for InitHandler {
    fn supports(&self, instruction_type: InstructionType) -> bool {
        matches!(instruction_type, InstructionType::Init)
    }

    async fn handle(&self, payload: &str, bot: &mut HanabLiveBot) -> Result<(), HandlerError> {
        let game_init_data: GameInitData = serde_json::from_str(payload)?;
        bot.on_game_init_data_received(game_init_data).await;
        Ok(())
    }

    fn next(&self) -> Option<&dyn Handler> {
        Some(&ChatMessageHandler)
    }
}
