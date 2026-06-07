use std::fs;
use std::path::Path;

use eel::bot::configuration::Configuration;
use eel::hanablive::bot::hanab_live_bot::HanabLiveBot;
use eel::hanablive::client::http_client::HttpClient;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

#[tokio::main]
async fn main() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    let log_dir = "/app/logs";
    if !Path::new(log_dir).exists() {
        let _ = fs::create_dir_all(log_dir);
    }
    let file_appender = tracing_appender::rolling::never(log_dir, "bot.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .with_filter(filter.clone()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_filter(filter),
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
