//! Helper functions for RabbitMQ management

use reqwest::Client;
use server::utils::generate_random_string;

pub const USER: &str = "guest";
pub const PASSWORD: &str = "123456";

pub async fn create_vhost(client: &Client, rmq_addr: &str, vhost_name: &str) -> anyhow::Result<()> {
    let url = format!(
        "{}/api/vhosts/{}",
        rmq_addr,
        urlencoding::encode(vhost_name)
    );
    let response = client
        .put(&url)
        .header("Content-Type", "application/json")
        .send()
        .await?;
    if !response.status().is_success() {
        eprintln!("{}", response.status());
        return Err(anyhow::anyhow!("Failed to create vhost"));
    }
    Ok(())
}

pub async fn delete_vhost(client: &Client, rmq_addr: &str, vhost_name: &str) {
    let url = format!(
        "{}/api/vhosts/{}",
        rmq_addr,
        urlencoding::encode(vhost_name)
    );
    client
        .delete(&url)
        .header("Content-Type", "application/json")
        .send()
        .await
        .unwrap();
}

pub async fn create_random_vhost(client: &Client, rmq_addr: &str) -> anyhow::Result<String> {
    let name = generate_random_string(20);
    create_vhost(client, rmq_addr, &name).await?;
    Ok(name)
}
