use eel::bot::configuration::Configuration;
use eel::hanablive::bot::hanab_live_bot::HanabLiveBot;
use eel::hanablive::client::http_client::HttpClient;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let username = std::env::var("EEL_BOT_USERNAME").expect("EEL_BOT_USERNAME must be set");
    let password = std::env::var("EEL_BOT_PASSWORD").expect("EEL_BOT_PASSWORD must be set");

    let http_client = HttpClient::new();
    let session_cookie = http_client
        .sign_in(&username, &password)
        .await
        .expect("Failed to connect to HanabLive");
    let mut bot = HanabLiveBot::new(Configuration::default(), &session_cookie).await;
    bot.run().await;
}
