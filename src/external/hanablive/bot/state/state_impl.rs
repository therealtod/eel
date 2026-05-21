use std::collections::HashMap;

use super::game_init_data_received::GameInitDataReceivedState;
use super::initial::InitialState;
use super::logged_in::LoggedInState;
use super::playing::PlayingState;
use super::table_joined::TableJoinedState;
use crate::external::hanablive::dto::instruction::game_action_list::GameAction;
use crate::external::hanablive::dto::instruction::game_action_list::GameActionListData;
use crate::external::hanablive::dto::table::Table;

pub enum BotState {
    Initial(InitialState),
    LoggedIn(LoggedInState),
    TableJoined(TableJoinedState),
    GameInitDataReceived(GameInitDataReceivedState),
    Playing(PlayingState),
}

impl BotState {
    pub fn set_tables(&mut self, tables: HashMap<usize, Table>) {
        match self {
            BotState::Initial(s) => s.common_state.tables = tables,
            BotState::LoggedIn(s) => s.common_state.tables = tables,
            BotState::TableJoined(s) => s.common_state.tables = tables,
            BotState::GameInitDataReceived(s) => s.common_state.tables = tables,
            BotState::Playing(s) => s.common_state.tables = tables.clone(),
        }
    }

    pub fn put_table(&mut self, table: Table) {
        match self {
            BotState::Initial(s) => {
                s.common_state.tables.insert(table.id, table);
            }
            BotState::LoggedIn(s) => {
                s.common_state.tables.insert(table.id, table);
            }
            BotState::TableJoined(s) => {
                s.common_state.tables.insert(table.id, table);
            }
            BotState::GameInitDataReceived(s) => {
                s.common_state.tables.insert(table.id, table);
            }
            BotState::Playing(s) => {
                s.common_state.tables.insert(table.id, table);
            }
        }
    }

    pub async fn on_game_action_list_received(
        &mut self,
        game_action_list_data: GameActionListData,
    ) -> Result<(), String> {
        match self {
            BotState::GameInitDataReceived(s) => {
                s.on_game_action_list_received(game_action_list_data).await
            }
            _ => Err(format!(
                "GameActionList should not be received in state: {}",
                self.variant_name()
            )),
        }
    }

    pub async fn on_game_action(&mut self, game_action: GameAction) -> Result<(), String> {
        match self {
            BotState::Playing(s) => s.on_game_action(game_action).await,
            _ => Err(format!(
                "GameAction should not be received in state: {}",
                self.variant_name()
            )),
        }
    }

    pub fn variant_name(&self) -> &'static str {
        match self {
            BotState::Initial(_) => "Initial",
            BotState::LoggedIn(_) => "LoggedIn",
            BotState::TableJoined(_) => "TableJoined",
            BotState::GameInitDataReceived(_) => "GameInitDataReceived",
            BotState::Playing(_) => "Playing",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("Invalid action: {0}")]
    InvalidAction(String),
}
