use reqwest::Client;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

use crate::external::hanablive::constants::{HOSTNAME, LOGIN_PATH};

pub struct HttpClient {
    client: Client,
    cookie: Arc<Mutex<Option<String>>>,
}

impl HttpClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("eel-bot")
            .build()
            .expect("failed to build HTTP client");

        Self {
            client,
            cookie: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<(), Error> {
        let url = format!("https://{HOSTNAME}{LOGIN_PATH}");
        let body = format!(
            "username={}&password={}&version=bot",
            urlencoding::encode(username),
            urlencoding::encode(password)
        );

        debug!("Logging in to {}", url);

        let response = self
            .client
            .post(&url)
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::LoginFailed(response.status()));
        }

        // Extract the session cookie from response headers
        if let Some(cookie_header) = response.headers().get(reqwest::header::SET_COOKIE) {
            let cookie_str = cookie_header.to_str().unwrap_or("");
            let session_cookie = cookie_str
                .split(';')
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            let mut cookie = self.cookie.lock().await;
            *cookie = Some(session_cookie);
            debug!("Received session cookie");
        }

        debug!("Login successful");
        Ok(())
    }

    pub async fn get_cookie(&self) -> Option<String> {
        self.cookie.lock().await.clone()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Login failed with status: {0}")]
    LoginFailed(reqwest::StatusCode),
}
