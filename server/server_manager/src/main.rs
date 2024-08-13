#[tokio::main]
async fn main() -> anyhow::Result<()>{
    server_manager::lib_main().await
}
