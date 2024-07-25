mod cfg;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    author = "SkyUOI",
    about = "The Server of OurChat",
)]
struct ArgsParser {
    #[arg(short, long, default_value_t = cfg::DEFAULT_PORT)]
    port:usize,
    #[arg(long, default_value_t = String::from(cfg::DEFAULT_IP))]
    ip: String
}

pub async fn lib_main() -> anyhow::Result<()> {
    let parser = ArgsParser::parse();
    let port = parser.port;
    let ip = parser.ip;
    Ok(())
}