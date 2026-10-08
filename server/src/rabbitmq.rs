use base::constants::ID;
use base::rabbitmq::http_server::VERIFY_QUEUE;
use deadpool_lapin::lapin::options::{ExchangeDeclareOptions, QueueDeclareOptions};
use deadpool_lapin::lapin::types::{FieldTable, ShortString};
use deadpool_lapin::lapin::{Channel, ExchangeKind};

pub const USER_MSG_DIRECT_EXCHANGE: &str = "user_msg";
pub const USER_MSG_BROADCAST_EXCHANGE: &str = "user_broadcast_msg";

// WebRTC signaling
pub const WEBRTC_SIGNAL_EXCHANGE: &str = "webrtc_signal";
pub const WEBRTC_FANOUT_EXCHANGE: &str = "webrtc_fanout";

pub async fn create_user_message_direct_exchange(channel: &Channel) -> anyhow::Result<()> {
    channel
        .exchange_declare(
            ShortString::from(USER_MSG_DIRECT_EXCHANGE),
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
            ShortString::from(USER_MSG_BROADCAST_EXCHANGE),
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
            ShortString::from(WEBRTC_SIGNAL_EXCHANGE),
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
            ShortString::from(WEBRTC_FANOUT_EXCHANGE),
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
            ShortString::from(VERIFY_QUEUE),
            QueueDeclareOptions {
                exclusive: true,
                auto_delete: true,
                ..Default::default()
            },
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
            ShortString::from(exchange_name.as_ref()),
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

/// Name of the per-STREAM queue each fetch_msgs connection consumes from.
///
/// Every stream gets its own queue (`{user_id}.{uuid}`) instead of sharing a
/// queue named after the user:
///
/// - an `exclusive` user-named queue can only be declared by one connection,
///   so a second stream for the same user (another device, or a test doing
///   back-to-back fetches while the previous pooled connection still holds
///   the queue) fails with 405 RESOURCE_LOCKED;
/// - exclusive queues are only deleted when their declaring CONNECTION
///   closes — pooled connections are returned, not closed, so the queue
///   lingered and kept the lock.
///
/// Per-stream queues are declared non-exclusive + auto-delete: RabbitMQ
/// removes such a queue once its last consumer goes away (the stream's
/// channel dropping ends the consumer), which is exactly the stream's
/// lifetime. Publishers keep addressing the user's routing key, and each
/// stream's queue is bound to it.
pub fn generate_stream_queue_name(user_id: ID) -> String {
    format!("{}.{}", user_id, uuid::Uuid::new_v4())
}

pub fn generate_route_key(user_id: ID) -> String {
    user_id.to_string()
}

pub fn generate_webrtc_route_key(user_id: ID) -> String {
    format!("webrtc:{}", user_id)
}
