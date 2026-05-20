use tokio::sync::mpsc;
use tracing::{debug, error, info};

use crate::external::hanablive::bot::state::BotState;
use crate::external::hanablive::bot::state::GameInitDataReceivedState;
use crate::external::hanablive::bot::state::initial::InitialState;
use crate::external::hanablive::bot::state::logged_in::LoggedInState;
use crate::external::hanablive::bot::state::table_joined::TableJoinedState;
use crate::external::hanablive::client::http_client::HttpClient;
use crate::external::hanablive::client::websocket_client::WebSocketClient;
use crate::external::hanablive::dto::game_init_data::GameInitData;
use crate::external::hanablive::dto::instruction::game_action_list::{
    GameAction, GameActionListData,
};
use crate::external::hanablive::dto::instruction_type::InstructionType;
use crate::external::hanablive::dto::outgoing::GetGameInfo1;
use crate::external::hanablive::dto::outgoing::Instruction;
use crate::external::hanablive::dto::table::Table;
use crate::external::hanablive::handler::{Handler, TableListHandler};

pub mod state;

pub enum BotEvent {
    SendInstruction(Box<dyn Instruction + Send>),
    TransitionToTableJoined(LoggedInState, usize),
    TransitionToGameInitDataReceived(TableJoinedState, GameInitData),
    TransitionToPlaying(GameInitDataReceivedState),
}

pub struct HanabLiveBot {
    pub state: BotState,
    http_client: HttpClient,
    sender: mpsc::UnboundedSender<BotEvent>,
    receiver: mpsc::UnboundedReceiver<BotEvent>,
}

impl HanabLiveBot {
    pub fn new(username: String, password: String) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let initial_state = InitialState::new(username.clone(), password.clone());

        Self {
            state: BotState::Initial(initial_state),
            http_client: HttpClient::new(),
            sender,
            receiver,
        }
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        let initial = match &self.state {
            BotState::Initial(s) => s,
            _ => return Err(Error::InvalidState("Expected Initial state".into())),
        };

        let username = initial.username.clone();
        let password = initial.password.clone();

        info!("Logging in as {}", username);
        self.http_client.login(&username, &password).await?;

        let cookie = self
            .http_client
            .get_cookie()
            .await
            .ok_or_else(|| Error::NoCookie)?;

        info!("Connecting to WebSocket");
        let mut ws = WebSocketClient::connect(&cookie).await?;

        let sender = self.sender.clone();

        if let BotState::Initial(initial) = std::mem::replace(
            &mut self.state,
            BotState::Initial(InitialState::new(String::new(), String::new())),
        ) {
            let logged_in = LoggedInState {
                common_state: initial.common_state,
                sender: sender.clone(),
            };
            self.state = BotState::LoggedIn(logged_in);
        }

        info!("Connected. Waiting for messages...");

        loop {
            tokio::select! {
                message = ws.receive() => {
                    match message {
                        Some(Ok(text)) => {
                            self.handle_message(&text, &mut ws).await?;
                        }
                        Some(Err(e)) => {
                            error!("WebSocket error: {}", e);
                            break;
                        }
                        None => {
                            info!("WebSocket closed");
                            break;
                        }
                    }
                }
                event = self.receiver.recv() => {
                    match event {
                        Some(BotEvent::SendInstruction(instruction)) => {
                            let msg = instruction.to_websocket_message();
                            if let Err(e) = ws.send(&msg).await {
                                error!("Failed to send message: {}", e);
                            }
                        }
                        Some(BotEvent::TransitionToTableJoined(logged_in, table_id)) => {
                            self.state = BotState::TableJoined(
                                logged_in.transition_to_table_joined(table_id)
                            );
                        }
                        Some(BotEvent::TransitionToGameInitDataReceived(table_joined, game_init_data)) => {
                            self.state = BotState::GameInitDataReceived(
                                table_joined.transition_to_game_init_data_received(game_init_data)
                            );
                        }
                        Some(BotEvent::TransitionToPlaying(game_init_data_received)) => {
                            self.state = BotState::Playing(
                                game_init_data_received.transition_to_playing()
                            );
                        }
                        None => break,
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_message(
        &mut self,
        message: &str,
        _ws: &mut WebSocketClient,
    ) -> Result<(), Error> {
        let tokens: Vec<&str> = message.splitn(2, ' ').collect();
        if tokens.len() < 2 {
            debug!("Ignoring malformed message: {}", message);
            return Ok(());
        }

        let message_type_str = tokens[0];
        let payload = tokens[1];

        let instruction_type = match InstructionType::from_string(message_type_str) {
            Some(t) => t,
            None => {
                debug!("Ignoring unknown instruction type: {}", message_type_str);
                return Ok(());
            }
        };

        debug!("Received: {:?}", instruction_type);

        match instruction_type {
            InstructionType::TableList => {
                TableListHandler.handle(payload, self).await?;
            }
            InstructionType::TableStart => {
                let tree: serde_json::Value = serde_json::from_str(payload)?;
                let table_id = tree["tableID"].as_u64().unwrap_or(0) as usize;
                self.send_instruction(GetGameInfo1 { table_id }).await;
            }
            InstructionType::Init => {
                let game_init_data: GameInitData = serde_json::from_str(payload)?;
                self.on_game_init_data_received(game_init_data).await;
            }
            InstructionType::GameActionList => {
                let game_action_list: GameActionListData = serde_json::from_str(payload)?;
                if let Err(e) = self
                    .state
                    .on_game_action_list_received(game_action_list)
                    .await
                {
                    error!("Error processing game action list: {}", e);
                }
            }
            InstructionType::GameAction => {
                let game_action: GameAction = serde_json::from_str(payload)?;
                if let Err(e) = self.state.on_game_action(game_action).await {
                    error!("Error processing game action: {}", e);
                }
            }
            InstructionType::Chat => {
                let _chat: crate::external::hanablive::dto::chat_message::ChatMessage =
                    serde_json::from_str(payload)?;
            }
            _ => {
                debug!("Unhandled instruction type: {:?}", instruction_type);
            }
        }

        Ok(())
    }

    pub async fn send_instruction<T: Instruction + Send + 'static>(&self, instruction: T) {
        let _ = self
            .sender
            .send(BotEvent::SendInstruction(Box::new(instruction)));
    }

    pub async fn set_tables(&mut self, tables: Vec<Table>) {
        let table_map: std::collections::HashMap<usize, Table> =
            tables.into_iter().map(|t| (t.id, t)).collect();
        self.state.set_tables(table_map);
    }

    pub async fn put_table(&mut self, table: Table) {
        self.state.put_table(table);
    }

    pub async fn on_game_init_data_received(&mut self, game_init_data: GameInitData) {
        info!(
            "Game init data received for table {}",
            game_init_data.table_id
        );

        if let BotState::TableJoined(table_joined) = std::mem::replace(
            &mut self.state,
            BotState::Initial(InitialState::new(String::new(), String::new())),
        ) {
            let _ = self.sender.send(BotEvent::TransitionToGameInitDataReceived(
                table_joined,
                game_init_data,
            ));
        }
    }

    pub async fn join_table(&mut self, table_id: usize) -> Result<(), Error> {
        if let BotState::LoggedIn(logged_in) = std::mem::replace(
            &mut self.state,
            BotState::Initial(InitialState::new(String::new(), String::new())),
        ) {
            let sender = logged_in.sender.clone();
            logged_in.join_table(table_id).await.map_err(Error::State)?;
            let logged_in2 = LoggedInState {
                common_state: logged_in.common_state,
                sender,
            };
            let _ = self
                .sender
                .send(BotEvent::TransitionToTableJoined(logged_in2, table_id));
        }
        Ok(())
    }

    pub async fn join_table_with_password(
        &mut self,
        table_id: usize,
        password: String,
    ) -> Result<(), Error> {
        if let BotState::LoggedIn(logged_in) = std::mem::replace(
            &mut self.state,
            BotState::Initial(InitialState::new(String::new(), String::new())),
        ) {
            let sender = logged_in.sender.clone();
            logged_in
                .join_table_with_password(table_id, password)
                .await
                .map_err(Error::State)?;
            let logged_in2 = LoggedInState {
                common_state: logged_in.common_state,
                sender,
            };
            let _ = self
                .sender
                .send(BotEvent::TransitionToTableJoined(logged_in2, table_id));
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] crate::external::hanablive::client::http_client::Error),
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] crate::external::hanablive::client::websocket_client::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Handler error: {0}")]
    Handler(#[from] crate::external::hanablive::handler::HandlerError),
    #[error("No session cookie received")]
    NoCookie,
    #[error("Invalid state: {0}")]
    InvalidState(String),
    #[error("State error: {0}")]
    State(String),
}
