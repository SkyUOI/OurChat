use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    server::lib_main().await?;
    Ok(())
}
