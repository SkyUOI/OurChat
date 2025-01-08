use clap::Parser;
use server::ArgsParser;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let parser = ArgsParser::parse();
    let config = server::get_configuration(parser.shared_cfg.config.clone())?;
    // let email_client = config.main_cfg.email.build_email_client().ok();
    let mut application = server::Application::build(parser, config).await?;
    application.run_forever().await?;
    tracing::info!("Application stopped");
    Ok(())
}
