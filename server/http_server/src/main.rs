use http_server::Launcher;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    color_eyre::install().ok();
    dotenvy::dotenv().ok();
    let mut app = Launcher::build().await?;
    app.run_forever().await?;
    Ok(())
}
