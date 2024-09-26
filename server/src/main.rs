use clap::Parser;
use server::ArgsParser;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let parser = ArgsParser::parse();
    let mut application = server::Application::build(parser, None).await?;
    application.run_forever().await
}
