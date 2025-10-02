use server::ARG_PARSER;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let config = server::get_configuration(ARG_PARSER.shared_cfg.config.clone())?;
    let mut application = server::Application::build(ARG_PARSER.clone(), config).await?;
    application.run_forever().await?;
    tracing::info!("Application stopped");
    Ok(())
}
