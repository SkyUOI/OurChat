use base::consts::ID;
use base::rabbitmq::http_server::VERIFY_QUEUE;
use deadpool_lapin::lapin::ExchangeKind;
use deadpool_lapin::lapin::options::{ExchangeDeclareOptions, QueueDeclareOptions};
use deadpool_lapin::lapin::types::FieldTable;

pub const USER_MSG_EXCHANGE: &str = "user_msg";

/// Init RabbitMQ
pub async fn init(rmq: &deadpool_lapin::Pool) -> anyhow::Result<()> {
    let connection = rmq.get().await?;
    let channel = connection.create_channel().await?;
    channel
        .exchange_declare(
            USER_MSG_EXCHANGE,
            ExchangeKind::Direct,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    // Declare the verify queue
    let channel = connection.create_channel().await?;
    channel
        .queue_declare(
            VERIFY_QUEUE,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    Ok(())
}

pub fn generate_client_name(user_id: ID) -> String {
    user_id.to_string()
}

pub fn generate_route_key(user_id: ID) -> String {
    user_id.to_string()
}
