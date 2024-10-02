use clap::Parser;
use server::ArgsParser;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let parser = ArgsParser::parse();
    let config = server::get_configuration(parser.shared_cfg.config.as_ref())?;
    let email_client = config.build_email_client().ok();
    let config = server::Cfg::new(config)?;
    let mut application = server::Application::build(parser, config, email_client).await?;
    application.run_forever().await
}
