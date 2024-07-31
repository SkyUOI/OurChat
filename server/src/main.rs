use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    log::info!("Server initing..");
    server::lib_main().await?;
    Ok(())
}
