//! Abstraction over the bot's outbound channel.
//!
//! The [`BotState`](crate::hanablive::bot::bot_state::BotState) handshake methods need to send a
//! handful of protocol acks (`getGameInfo2`, `loaded`, `tableUnattend`, the chosen action). In
//! production that channel is the live [`WebSocketClient`]; for offline transcript replay and
//! tests there is no socket, so [`NullSink`] discards the acks while the same state-machine code
//! runs unchanged. `async-trait` is used so the methods can take `&mut dyn OutboundSink`.

use async_trait::async_trait;

use crate::hanablive::client::websocket_client::{Error, WebSocketClient};
use crate::hanablive::outbound_websocket_message::OutboundWebsocketMessage;

/// The bot's outbound message channel. Implemented by the live [`WebSocketClient`] and by the
/// no-op [`NullSink`].
#[async_trait]
pub trait OutboundSink {
    /// Send an outbound message. Implementations that have no real connection (tests, replay)
    /// return `Ok(())` without doing anything.
    async fn send(&mut self, msg: &OutboundWebsocketMessage) -> Result<(), Error>;
}

#[async_trait]
impl OutboundSink for WebSocketClient {
    async fn send(&mut self, msg: &OutboundWebsocketMessage) -> Result<(), Error> {
        // Disambiguate to the inherent method (inherent methods take priority over trait
        // methods of the same name, so this is not self-recursion).
        WebSocketClient::send(self, msg).await
    }
}

/// An [`OutboundSink`] that discards every message. Used to drive the state machine offline —
/// e.g. replaying a recorded liveplay transcript — where the protocol acks are irrelevant.
pub struct NullSink;

#[async_trait]
impl OutboundSink for NullSink {
    async fn send(&mut self, _msg: &OutboundWebsocketMessage) -> Result<(), Error> {
        Ok(())
    }
}
