use clap::Parser;
use tracing_subscriber::EnvFilter;
mod game;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    name: String,

    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}
