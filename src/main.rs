use bot::Bot;
use dotenvy::dotenv;
use opts::Opts;
use clap::Parser;
use color_eyre::Result;
use tracing_subscriber;

pub mod opts;
mod bot;
mod errors;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>  {
    tracing::info!("Starting!");
    // initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "twitchbot=debug".into()),
        )
        .init();
    color_eyre::install().unwrap();
    dotenv().expect(".env file not found");
    let args = Opts::parse();

    let mut bot = Bot::new(args);
    let _res = bot.run().await;

    Ok(())
}
