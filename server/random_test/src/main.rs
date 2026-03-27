mod actions;
mod config;
mod framework;
mod generator;
mod metrics;
mod state;
mod streaming;
mod validators;

use base::setting::read_config_and_deserialize;
use client::oc_helper::client::ClientCoreConfig;
use framework::RandomTestEngine;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Get config file path from command line argument
    let args: Vec<String> = std::env::args().collect();
    let config_path = if args.len() > 1 {
        args[1].clone()
    } else {
        "random_test.toml".to_string()
    };

    // Load .env file if present
    dotenvy::dotenv().ok();

    // Load configuration using server's config reading approach
    let config: config::RandomTestConfig = read_config_and_deserialize(&config_path)?;

    // Initialize logging
    base::log::logger_init(config.verbose, None, std::io::stdout, "random_test");

    info!("═══════════════════════════════════════════════════════════════");
    info!("🎲 OurChat Random Test");
    info!("═══════════════════════════════════════════════════════════════");
    info!("Configuration:");
    info!("  Users: {}", config.num_users);
    info!("  Duration: {:?}", config.running_duration);
    info!("  Action Rate: {}/sec", config.actions_per_second);
    info!("  Concurrency: {}", config.concurrency);
    info!("  Seed: {}", config.seed);

    // Create client core with the config
    let client_core = create_client_core(&config).await?;

    // Create and run the test engine
    let mut engine = RandomTestEngine::new(config, client_core)?;
    let result = engine.run().await;

    // Always print the final report
    engine.print_final_report().await;

    // Cleanup on exit
    engine.cleanup().await;

    result
}

/// Create a ClientCore connected to the server
async fn create_client_core(
    config: &config::RandomTestConfig,
) -> anyhow::Result<client::ClientCore> {
    let port = config.port;
    let ip = &config.ip;

    let client_config = ClientCoreConfig {
        ip: ip.to_owned(),
        port,
        enable_ssl: None,
    };

    client::ClientCore::new(client_config).await
}
