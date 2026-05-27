#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let username = std::env::var("EEL_BOT_USERNAME")
        .expect("EEL_BOT_USERNAME must be set");
    let password = std::env::var("EEL_BOT_PASSWORD")
        .expect("EEL_BOT_PASSWORD must be set");
}