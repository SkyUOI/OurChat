use base::constants::ID;
use base::rabbitmq::http_server::VERIFY_QUEUE;
use deadpool_lapin::lapin::options::{ExchangeDeclareOptions, QueueDeclareOptions};
use deadpool_lapin::lapin::types::FieldTable;
use deadpool_lapin::lapin::{Channel, ExchangeKind};

pub const USER_MSG_DIRECT_EXCHANGE: &str = "user_msg";
pub const USER_MSG_BROADCAST_EXCHANGE: &str = "user_broadcast_msg";

// WebRTC signaling
pub const WEBRTC_SIGNAL_EXCHANGE: &str = "webrtc_signal";
pub const WEBRTC_FANOUT_EXCHANGE: &str = "webrtc_fanout";

pub async fn create_user_message_direct_exchange(channel: &Channel) -> anyhow::Result<()> {
    channel
        .exchange_declare(
            USER_MSG_DIRECT_EXCHANGE,
            ExchangeKind::Direct,
            ExchangeDeclareOptions {
                auto_delete: false,
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;
    Ok(())
}

pub async fn create_user_message_broadcast_exchange(channel: &Channel) -> anyhow::Result<()> {
    channel
        .exchange_declare(
            USER_MSG_BROADCAST_EXCHANGE,
            ExchangeKind::Fanout,
            ExchangeDeclareOptions {
                auto_delete: false,
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;
    Ok(())
}

pub async fn create_webrtc_signal_exchange(channel: &Channel) -> anyhow::Result<()> {
    channel
        .exchange_declare(
            WEBRTC_SIGNAL_EXCHANGE,
            ExchangeKind::Direct,
            ExchangeDeclareOptions {
                auto_delete: false,
                durable: false,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;
    Ok(())
}

pub async fn create_webrtc_fanout_exchange(channel: &Channel) -> anyhow::Result<()> {
    channel
        .exchange_declare(
            WEBRTC_FANOUT_EXCHANGE,
            ExchangeKind::Fanout,
            ExchangeDeclareOptions {
                auto_delete: false,
                durable: false,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;
    Ok(())
}

/// Init RabbitMQ
pub async fn init(rmq: &deadpool_lapin::Pool) -> anyhow::Result<()> {
    let connection = rmq.get().await?;
    let channel = connection.create_channel().await?;
    create_user_message_direct_exchange(&channel).await?;
    create_user_message_broadcast_exchange(&channel).await?;
    create_webrtc_signal_exchange(&channel).await?;
    create_webrtc_fanout_exchange(&channel).await?;
    // Declare the verify queue
    channel
        .queue_declare(
            VERIFY_QUEUE,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    Ok(())
}

pub async fn check_exchange_exist(
    channel: &Channel,
    kind: ExchangeKind,
    exchange_name: impl AsRef<str>,
) -> anyhow::Result<()> {
    channel
        .exchange_declare(
            exchange_name.as_ref(),
            kind,
            ExchangeDeclareOptions {
                passive: true,
                ..Default::default()
            },
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

pub fn generate_webrtc_route_key(user_id: ID) -> String {
    format!("webrtc:{}", user_id)
}
