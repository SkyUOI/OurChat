#[tokio::main]
async fn main() -> anyhow::Result<()> {
    static_keys::global_init();
    server::lib_main().await
}
