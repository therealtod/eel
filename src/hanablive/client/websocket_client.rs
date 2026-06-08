use std::time::Duration;

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::time::timeout;
use tokio_tungstenite::{Connector, connect_async_tls_with_config, tungstenite::Message};
use tracing::{debug, error, info, warn};

use crate::hanablive::constants::{HOSTNAME, WEBSOCKET_PATH};
use crate::hanablive::inbound_websocket_message::InboundWebsocketMessage;
use crate::hanablive::liveplay_recorder::LiveplayRecorder;
use crate::hanablive::outbound_websocket_message::OutboundWebsocketMessage;

type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// If no frame at all — including the server's keepalive pings — arrives within this window, the
/// connection is presumed dead and the read loop exits so the bot can reconnect. hanab.live sends
/// pings well inside this interval even when no game data is flowing, so prolonged silence is a
/// reliable signal of a half-open socket that a plain blocking `read.next()` would never surface.
const READ_TIMEOUT: Duration = Duration::from_secs(90);

pub struct WebSocketClient {
    write: SplitSink<WsStream, Message>,
    /// Cleared the first time a [`Self::send`] fails. Send errors are otherwise logged and
    /// swallowed by the `bot_state` handlers, which would leave the engine diverged from the
    /// server with no recovery path. The run loop polls [`Self::is_connected`] and treats a
    /// flipped flag as a disconnect, driving a reconnect (and the state reset that resyncs us).
    connected: bool,
}

impl WebSocketClient {
    /// Connect to the hanab.live WebSocket endpoint.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Request`] if the request builder fails, or [`Error::WebSocket`] if the
    /// WebSocket handshake fails.
    pub async fn connect(
        cookie: &str,
    ) -> Result<(Self, mpsc::Receiver<InboundWebsocketMessage>), Error> {
        let url = format!("wss://{HOSTNAME}{WEBSOCKET_PATH}");
        debug!("Connecting to WebSocket at {}", url);

        let root_store = rustls::RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
        };

        let config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let connector = Connector::Rustls(std::sync::Arc::new(config));

        let request = tokio_tungstenite::tungstenite::http::Request::builder()
            .uri(&url)
            .header("Host", HOSTNAME)
            .header("Cookie", cookie)
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header(
                "Sec-WebSocket-Key",
                tokio_tungstenite::tungstenite::handshake::client::generate_key(),
            )
            .body(())?;

        let (stream, response) =
            connect_async_tls_with_config(request, None, true, Some(connector)).await?;
        debug!("WebSocket connected, status: {}", response.status());

        let (write, read) = stream.split();
        let (message_tx, message_rx) = mpsc::channel(1000);

        // Debug builds capture the raw inbound stream for offline replay; `new` returns `None`
        // (and the recorder compiles to nothing) in optimized builds. See [`LiveplayRecorder`].
        let recorder = LiveplayRecorder::new();
        tokio::spawn(Self::read_loop(read, message_tx, recorder));

        Ok((
            Self {
                write,
                connected: true,
            },
            message_rx,
        ))
    }

    async fn read_loop(
        mut read: SplitStream<WsStream>,
        message_tx: mpsc::Sender<InboundWebsocketMessage>,
        recorder: Option<LiveplayRecorder>,
    ) {
        loop {
            let next = match timeout(READ_TIMEOUT, read.next()).await {
                Ok(next) => next,
                Err(_elapsed) => {
                    warn!("No WebSocket frame within {READ_TIMEOUT:?}; presuming connection dead");
                    break;
                }
            };
            match next {
                Some(Ok(Message::Text(text))) => {
                    if let Some(recorder) = &recorder {
                        recorder.record(&text);
                    }
                    if let Err(e) = message_tx.send(InboundWebsocketMessage::parse(&text)).await {
                        error!("Failed to broadcast message: {}", e);
                    }
                }
                Some(Ok(Message::Binary(_))) => warn!("Received binary message, ignoring."),
                Some(Ok(Message::Close(_))) => {
                    info!("WebSocket closed");
                    break;
                }
                Some(Ok(Message::Pong(_) | _)) => {}
                Some(Err(e)) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                None => {
                    info!("WebSocket stream ended");
                    break;
                }
            }
        }
        // Falling out of the loop drops `message_tx`; once the last sender is gone the bot's
        // `message_rx.recv()` returns `None`, which the run loop treats as a disconnect and uses
        // to drive reconnection.
    }

    /// Send an outbound message over the WebSocket.
    ///
    /// On failure the connection is marked dead (see [`Self::is_connected`]) so a swallowed send
    /// error still surfaces as a disconnect to the run loop.
    ///
    /// # Errors
    ///
    /// Returns [`Error::WebSocket`] if the write fails.
    pub async fn send(&mut self, msg: &OutboundWebsocketMessage) -> Result<(), Error> {
        let text = msg.serialize();
        if let Err(e) = self.write.send(Message::from(text)).await {
            self.connected = false;
            return Err(e.into());
        }
        Ok(())
    }

    /// Whether the connection is still believed healthy. Returns `false` once any [`Self::send`]
    /// has failed, signalling the run loop to drop the connection and reconnect/resync.
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Create an in-memory WebSocket client for unit tests. All messages sent through this client
    /// are silently discarded by a local server task; sends always succeed.
    #[cfg(test)]
    pub(crate) async fn for_test() -> Self {
        use tokio::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                if let Ok(ws) = tokio_tungstenite::accept_async(stream).await {
                    let (_, mut read) = futures_util::StreamExt::split(ws);
                    while futures_util::StreamExt::next(&mut read).await.is_some() {}
                }
            }
        });

        let url = format!("ws://127.0.0.1:{}/", addr.port());
        let (ws, _) = tokio_tungstenite::connect_async(url).await.unwrap();
        let (write, _) = futures_util::StreamExt::split(ws);
        Self {
            write,
            connected: true,
        }
    }

    /// Reconnect to the hanab.live WebSocket endpoint with exponential backoff.
    ///
    /// Tries up to `max_attempts` times, doubling the backoff each time (up to `max_backoff`).
    /// On success, returns the new client and message receiver.
    ///
    /// # Errors
    ///
    /// Returns an error if all reconnection attempts fail.
    pub async fn reconnect(
        cookie: &str,
        max_attempts: usize,
        initial_backoff: Duration,
        max_backoff: Duration,
    ) -> Result<(Self, mpsc::Receiver<InboundWebsocketMessage>), Error> {
        let mut backoff = initial_backoff;
        for attempt in 1..=max_attempts {
            debug!(
                "Reconnect attempt {attempt}/{max_attempts} in {backoff:?}",
            );
            tokio::time::sleep(backoff).await;
            match Self::connect(cookie).await {
                Ok((client, rx)) => {
                    info!("Reconnected to hanab.live");
                    return Ok((client, rx));
                }
                Err(e) => {
                    warn!("Reconnect attempt {attempt} failed: {e}");
                    backoff = next_backoff(backoff, max_backoff);
                }
            }
        }
        Err(Error::ReconnectFailed(max_attempts))
    }
}

/// Double the backoff, saturating at `max`.
fn next_backoff(current: Duration, max: Duration) -> Duration {
    current.saturating_mul(2).min(max)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("Request error: {0}")]
    Request(#[from] tokio_tungstenite::tungstenite::http::Error),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("Reconnection failed after {0} attempts")]
    ReconnectFailed(usize),
}
