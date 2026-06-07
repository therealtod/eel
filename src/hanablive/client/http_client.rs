use crate::hanablive::constants::{HOSTNAME, LOGIN_PATH};
use reqwest::Client;
use tracing::debug;

pub struct HttpClient {
    client: Client,
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClient {
    /// Create a new HTTP client.
    ///
    /// # Panics
    ///
    /// Panics if the reqwest client builder fails to build.
    #[must_use]
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("eel-bot")
            .build()
            .expect("failed to build HTTP client");

        Self { client }
    }

    /// Log in to hanab.live and store the session cookie.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the request fails, or [`Error::LoginFailed`] if the server
    /// returns a non-success status.
    pub async fn sign_in(&self, username: &str, password: &str) -> Result<String, Error> {
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
            debug!("Received session cookie");
            let cookie_str = cookie_header.to_str().unwrap_or("");
            let session_cookie = cookie_str
                .split(';')
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            return Ok(session_cookie);
        }

        Err(Error::MissingSessionCookie)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Login failed with status: {0}")]
    LoginFailed(reqwest::StatusCode),
    #[error("Missing session cookie in response headers")]
    MissingSessionCookie,
}
