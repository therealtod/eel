use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{Connector, connect_async_tls_with_config, tungstenite::Message};
use tracing::{debug, error};

use crate::external::hanablive::constants::{HOSTNAME, WEBSOCKET_PATH};

pub struct WebSocketClient {
    stream: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
}

impl WebSocketClient {
    pub async fn connect(cookie: &str) -> Result<Self, Error> {
        let url = format!("wss://{HOSTNAME}{WEBSOCKET_PATH}");
        debug!("Connecting to WebSocket at {}", url);

        let root_store = rustls::RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.iter().cloned().collect(),
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

        Ok(Self { stream })
    }

    pub async fn send(&mut self, message: &str) -> Result<(), Error> {
        debug!("Sending WS message: {}", message);
        self.stream.send(Message::Text(message.into())).await?;
        Ok(())
    }

    pub async fn receive(&mut self) -> Option<Result<String, Error>> {
        match self.stream.next().await {
            Some(Ok(Message::Text(text))) => Some(Ok(text.to_string())),
            Some(Ok(Message::Binary(data))) => match String::from_utf8(data.to_vec()) {
                Ok(text) => Some(Ok(text)),
                Err(e) => Some(Err(Error::Utf8(e))),
            },
            Some(Ok(Message::Close(_))) => {
                debug!("WebSocket closed");
                None
            }
            Some(Ok(_)) => None,
            Some(Err(e)) => {
                error!("WebSocket error: {}", e);
                Some(Err(Error::WebSocket(e)))
            }
            None => None,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),
    #[error("Request error: {0}")]
    Request(#[from] tokio_tungstenite::tungstenite::http::Error),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}
