mod chat;
mod init;
mod noop;
mod table;
mod table_list;
mod table_start;
mod welcome;

pub use chat::ChatMessageHandler;
pub use init::InitHandler;
pub use noop::NoOpHandler;
pub use table::TableHandler;
pub use table_list::TableListHandler;
pub use table_start::TableStartHandler;
pub use welcome::WelcomeHandler;

use async_trait::async_trait;

use crate::external::hanablive::bot::HanabLiveBot;
use crate::external::hanablive::dto::instruction_type::InstructionType;

#[async_trait]
pub trait Handler: Send + Sync {
    fn supports(&self, instruction_type: InstructionType) -> bool;
    async fn handle(&self, payload: &str, bot: &mut HanabLiveBot) -> Result<(), HandlerError>;
    fn next(&self) -> Option<&dyn Handler>;

    async fn dispatch(
        &self,
        instruction_type: InstructionType,
        payload: &str,
        bot: &mut HanabLiveBot,
    ) -> Result<(), HandlerError> {
        if self.supports(instruction_type) {
            self.handle(payload, bot).await
        } else if let Some(next) = self.next() {
            next.dispatch(instruction_type, payload, bot).await
        } else {
            tracing::debug!(
                "Unhandled message of type: {:?}\n{}",
                instruction_type,
                payload
            );
            Ok(())
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HandlerError {
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("State error: {0}")]
    State(String),
}
