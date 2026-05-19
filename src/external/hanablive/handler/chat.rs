use async_trait::async_trait;

use super::Handler;
use crate::external::hanablive::bot::HanabLiveBot;
use crate::external::hanablive::dto::chat_message::ChatMessage;
use crate::external::hanablive::dto::instruction_type::InstructionType;
use crate::external::hanablive::handler::{HandlerError, NoOpHandler};

pub struct ChatMessageHandler;

#[async_trait]
impl Handler for ChatMessageHandler {
    fn supports(&self, instruction_type: InstructionType) -> bool {
        matches!(instruction_type, InstructionType::Chat)
    }

    async fn handle(&self, payload: &str, _bot: &mut HanabLiveBot) -> Result<(), HandlerError> {
        let chat_message: ChatMessage = serde_json::from_str(payload)?;
        let tokens: Vec<&str> = chat_message.msg.split_whitespace().collect();
        if tokens.len() > 1 && tokens[0] == "!" {
            let command = tokens[1];
            let _args = &tokens[2..];
            tracing::info!("Received command: {} from {}", command, chat_message.who);
        }
        Ok(())
    }

    fn next(&self) -> Option<&dyn Handler> {
        Some(&NoOpHandler)
    }
}
