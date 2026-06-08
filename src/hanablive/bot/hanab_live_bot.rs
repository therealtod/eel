use crate::bot::USER_COMMAND_PREFIX;
use crate::bot::command::user_command::UserCommand;
use crate::bot::configuration::Configuration;
use crate::engine::convention::hgroup::h_group_convention_set::HGroupConventionSet;
use crate::engine::tree_action_selection_strategy::TreeActionSelectionStrategy;
use crate::game::variant::test_variants::NO_VARIANT;
use crate::hanablive::bot::bot_state::{BotState, TurnOutcome};
use crate::hanablive::client::websocket_client::WebSocketClient;
use crate::hanablive::dto::payload::chat::Chat;
use crate::hanablive::inbound_websocket_message::InboundWebsocketMessage;
use tokio::sync::mpsc;

pub struct HanabLiveBot {
    configuration: Configuration,
    session_cookie: String,
    state: BotState,
    websocket_client: WebSocketClient,
    message_rx: mpsc::Receiver<InboundWebsocketMessage>,
    convention_set: HGroupConventionSet,
    strategy: TreeActionSelectionStrategy,
}

impl HanabLiveBot {
    #[must_use]
    pub async fn new(configuration: Configuration, session_cookie: &str) -> Self {
        let (websocket_client, rx) = WebSocketClient::connect(session_cookie)
            .await
            .expect("Failed to connect to WebSocket");
        let convention_set =
            HGroupConventionSet::for_level(configuration.hgroup_level, &NO_VARIANT);
        Self {
            configuration,
            session_cookie: session_cookie.to_string(),
            state: BotState::initial_state(),
            websocket_client,
            message_rx: rx,
            convention_set,
            strategy: TreeActionSelectionStrategy::default(),
        }
    }

    /// Run the bot, reconnecting with exponential backoff whenever the WebSocket drops.
    ///
    /// [`Self::process_messages`] returns when the inbound stream ends (a close frame, a read
    /// error, or the keepalive timeout in the client's read loop). Each return is treated as a
    /// disconnect and drives a reconnect attempt; the loop only exits once reconnection gives up.
    pub async fn run(&mut self) {
        loop {
            self.process_messages().await;
            tracing::warn!("Inbound message stream ended; connection lost");
            match WebSocketClient::reconnect(
                &self.session_cookie,
                self.configuration.max_reconnect_attempts as usize,
                self.configuration.reconnect_initial_backoff,
                self.configuration.reconnect_max_backoff,
            )
            .await
            {
                Ok((websocket_client, message_rx)) => {
                    tracing::info!("Reconnected to hanab.live");
                    self.websocket_client = websocket_client;
                    self.message_rx = message_rx;
                    self.state = BotState::initial_state();
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to reconnect after {} attempts; shutting down: {e}",
                        self.configuration.max_reconnect_attempts
                    );
                    break;
                }
            }
        }
    }

    /// Process inbound messages until the stream ends (disconnect), then return.
    async fn process_messages(&mut self) {
        loop {
            let Some(message) = self.message_rx.recv().await else {
                return;
            };
            tracing::info!("Received message of type: {:?}", message.label());
            tracing::info!("Raw message: {:?}", message);
            match message {
                InboundWebsocketMessage::Welcome(welcome) => {
                    tracing::info!("Logged in as {}", welcome.username);
                }
                InboundWebsocketMessage::Chat(chat) => self.handle_chat(&chat).await,
                InboundWebsocketMessage::TableList(tables) => {
                    self.state.on_table_list_received(tables);
                }
                InboundWebsocketMessage::Table(table) => self.state.on_table_added(table),
                InboundWebsocketMessage::TableGone => {}
                InboundWebsocketMessage::TableStart(table_start) => {
                    self.state
                        .on_table_start(table_start.table_id, &mut self.websocket_client)
                        .await;
                }
                InboundWebsocketMessage::Init(init_data) => {
                    self.state
                        .on_init_received(init_data, &mut self.websocket_client)
                        .await;
                }
                InboundWebsocketMessage::CardIdentities(payload) => {
                    let outcome = self
                        .state
                        .on_card_identities_received(payload, &mut self.websocket_client)
                        .await;
                    self.handle_turn_outcome(outcome).await;
                }
                InboundWebsocketMessage::GameActionList(payload) => {
                    let outcome = self
                        .state
                        .on_game_action_list_received(
                            &payload,
                            &self.convention_set,
                            &mut self.websocket_client,
                        )
                        .await;
                    self.handle_turn_outcome(outcome).await;
                }
                InboundWebsocketMessage::GameAction(payload) => {
                    let outcome = self.state.apply_game_action(&payload, &self.convention_set);
                    self.handle_turn_outcome(outcome).await;
                }
                // Server-side warnings and messages we couldn't parse are the clearest signals
                // of protocol drift, so surface them at `warn` rather than the `debug` catch-all.
                InboundWebsocketMessage::Warning => {
                    tracing::warn!("Received warning message from server");
                }
                InboundWebsocketMessage::Unknown => {
                    tracing::warn!("Received unknown/unparseable message; possible protocol drift");
                }
                _ => tracing::debug!("Unhandled message type: {:?}", message),
            }

            // The state handlers above log and swallow send errors, so a failed send leaves the
            // engine diverged from the server. Surface it here as a disconnect: returning drives a
            // reconnect, which resets the bot to `initial_state` and resyncs from the server's
            // re-sent welcome/table list rather than playing on a diverged view.
            if !self.websocket_client.is_connected() {
                tracing::warn!(
                    "Send failure detected; dropping connection to reconnect and resync"
                );
                return;
            }
        }
    }

    async fn handle_turn_outcome(&mut self, outcome: TurnOutcome) {
        match outcome {
            TurnOutcome::OurTurn => {
                self.state
                    .take_turn(
                        &self.convention_set,
                        &self.strategy,
                        &mut self.websocket_client,
                    )
                    .await;
            }
            TurnOutcome::GameOver => {
                self.state.on_game_over(&mut self.websocket_client).await;
            }
            TurnOutcome::Waiting => {}
        }
    }

    async fn handle_chat(&mut self, chat: &Chat) {
        tracing::debug!("Received chat message: {:?}", chat);
        if chat.msg.starts_with(USER_COMMAND_PREFIX) {
            match UserCommand::parse(&chat.msg) {
                Ok(command) => match command {
                    UserCommand::JoinMe(password) => self.handle_join_me(&chat.who, password).await,
                    UserCommand::Leave => self.handle_leave().await,
                    // #11: these are recognized but not yet wired up. Log at info so a user
                    // issuing them gets a debuggable trail instead of silent no-ops.
                    UserCommand::JoinTable(..) => {
                        tracing::info!("!joinTable from {} is not yet implemented", chat.who);
                    }
                    UserCommand::SpectateTable(_) => {
                        tracing::info!("!spectateTable from {} is not yet implemented", chat.who);
                    }
                    UserCommand::StartGame => {
                        tracing::info!("!startGame from {} is not yet implemented", chat.who);
                    }
                },
                Err(e) => tracing::warn!("Failed to parse command: {e}"),
            }
        }
    }

    async fn handle_join_me(&mut self, sender: &str, password: Option<String>) {
        let Some(table_id) = self.state.find_table_for_user(sender) else {
            tracing::warn!("JoinMe from {sender}: user not found or not at a table");
            return;
        };
        tracing::info!("JoinMe from {sender}: joining table {table_id}");
        self.state
            .on_table_join_request(table_id, password, &mut self.websocket_client)
            .await;
    }

    async fn handle_leave(&mut self) {
        tracing::info!("Received leave command, leaving table");
        self.state
            .on_table_leave_request(&mut self.websocket_client)
            .await;
    }
}
