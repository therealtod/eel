use crate::external::hanablive::bot::BotEvent;
use crate::external::hanablive::bot::state::common::CommonState;
use crate::external::hanablive::bot::state::playing::PlayingState;
use crate::external::hanablive::dto::game_init_data::GameInitData;
use crate::external::hanablive::dto::instruction::game_action_list::GameActionListData;

pub struct GameInitDataReceivedState {
    pub common_state: CommonState,
    pub table_id: usize,
    pub game_init_data: GameInitData,
    pub sender: tokio::sync::mpsc::UnboundedSender<BotEvent>,
}

impl GameInitDataReceivedState {
    pub async fn on_game_action_list_received(
        &mut self,
        _game_action_list_data: GameActionListData,
    ) -> Result<(), String> {
        // Build the game POV and transition to PlayingState
        // This requires significant game state initialization
        Ok(())
    }

    pub fn transition_to_playing(self) -> PlayingState {
        PlayingState {
            common_state: self.common_state,
            table_id: self.table_id,
            game_init_data: self.game_init_data,
            sender: self.sender,
            actions_buffer: Vec::new(),
        }
    }
}
