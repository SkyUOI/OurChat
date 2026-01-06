mod framework;
mod tests;

use clap::Parser;

// Re-export for use in test modules
pub use tests::UsersGroup;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[derive(Debug, Parser, Default)]
#[command(author = "SkyUOI", about = "The Stress Test of OurChat")]
pub struct ArgsParser {
    #[arg(short, long, help = "The path of server config")]
    pub config: String,

    #[arg(
        long,
        help = "Filter tests by pattern (e.g., 'auth', '*msg*', 'session/*')",
        value_name = "PATTERN"
    )]
    pub filter: Vec<String>,

    #[arg(long, help = "Exclude tests by pattern", value_name = "PATTERN")]
    pub exclude: Vec<String>,

    #[arg(long, help = "List all available tests without running them")]
    pub list: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let args = ArgsParser::parse();

    // Handle --list flag
    // Note: All tests are automatically registered via #[ctor] at program startup
    if args.list {
        println!("{}", tests::registry::list_all_tests());
        return Ok(());
    }

    let mut app = {
        base::log::logger_init(true, None, std::io::stdout, "ourchat");
        let cfg = base::setting::read_config_and_deserialize(&args.config)?;
        client::ClientCore::new(cfg).await?
    };

    // Run filtered tests
    tests::registry::run_filtered_tests(&args.filter, &args.exclude, &mut app).await?;
    Ok(())
}
