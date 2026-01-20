use server::{ARG_PARSER, RUN_AS_STANDALONE};

// Used for performance improvement
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    server::global_init();
    let dotenv_err = dotenvy::dotenv();
    *RUN_AS_STANDALONE.lock() = true;
    let config = server::get_configuration(ARG_PARSER.shared_cfg.config.clone())?;
    let mut application = server::Application::build(ARG_PARSER.clone(), config).await?;
    if let Err(e) = dotenv_err {
        tracing::warn!(".env file is not loaded: {:?}", e)
    }
    application.run_forever().await?;
    tracing::info!("Application stopped");
    Ok(())
}
