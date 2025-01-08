use http_server::Launcher;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let mut app = Launcher::build().await?;
    app.run_forever().await?;
    Ok(())
}
