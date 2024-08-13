#[tokio::main]
async fn main() -> anyhow::Result<()> {
    server::lib_main().await
}
