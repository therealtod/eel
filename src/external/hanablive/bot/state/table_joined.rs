use crate::external::hanablive::bot::BotEvent;
use crate::external::hanablive::bot::state::common::CommonState;
use crate::external::hanablive::bot::state::game_init_data_received::GameInitDataReceivedState;
use crate::external::hanablive::dto::game_init_data::GameInitData;

#[derive(Debug)]
pub struct TableJoinedState {
    pub common_state: CommonState,
    pub table_id: usize,
    pub sender: tokio::sync::mpsc::UnboundedSender<BotEvent>,
}

impl TableJoinedState {
    pub async fn on_game_init_data_received(
        &mut self,
        _game_init_data: GameInitData,
    ) -> Result<(), String> {
        // Transition to GameInitDataReceivedState is handled by the bot
        // since it needs to update the state enum
        Ok(())
    }

    pub fn transition_to_game_init_data_received(
        self,
        game_init_data: GameInitData,
    ) -> GameInitDataReceivedState {
        GameInitDataReceivedState {
            common_state: self.common_state,
            table_id: self.table_id,
            game_init_data,
            sender: self.sender,
        }
    }
}
