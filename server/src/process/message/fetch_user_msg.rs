use std::time::Duration;

use crate::{
    db,
    process::error_msg::{SERVER_ERROR, TIME_FORMAT_ERROR, TIME_MISSING},
    rabbitmq::{create_user_message_broadcast_exchange, create_user_message_direct_exchange},
    server::{FetchMsgsStream, RpcServer},
};
use anyhow::Context;
use base::constants::ID;
use deadpool_lapin::lapin::options::{QueueBindOptions, QueueDeclareOptions};
use deadpool_lapin::lapin::types::FieldTable;
use pb::{
    service::ourchat::msg_delivery::v1::{
        FetchMsgsRequest, FetchMsgsResponse, fetch_msgs_response::RespondEventType,
    },
    time::TimeStampUtc,
};
use prost::Message;
use tokio::{select, sync::mpsc};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Response, Status};

pub async fn fetch_user_msg(
    server: &RpcServer,
    id: ID,
    request: tonic::Request<FetchMsgsRequest>,
) -> Result<Response<FetchMsgsStream>, Status> {
    match fetch_user_msg_impl(server, id, request).await {
        Ok(d) => Ok(d),
        Err(e) => match e {
            FetchMsgError::Db(_) | FetchMsgError::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            FetchMsgError::Status(s) => Err(s),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum FetchMsgError {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
}

async fn fetch_user_msg_impl(
    server: &RpcServer,
    id: ID,
    request: tonic::Request<FetchMsgsRequest>,
) -> Result<Response<FetchMsgsStream>, FetchMsgError> {
    let request = request.into_inner();
    let announcement_only = request.announcement_only;
    let history_limit = request.history_limit;
    let time: TimeStampUtc = match match request.time {
        Some(t) => t,
        None => {
            return Err(Status::invalid_argument(TIME_MISSING))?;
        }
    }
    .try_into()
    {
        Ok(t) => t,
        Err(_) => {
            return Err(Status::invalid_argument(TIME_FORMAT_ERROR))?;
        }
    };
    let (tx, rx) = mpsc::channel(32);
    let db_conn = server.db.clone();
    let fetch_page_size = server.shared_data.cfg().main_cfg.db.fetch_msg_page_size;
    let connection = server.get_rabbitmq_manager().await?;

    // Track active connection
    metrics::gauge!("active_connections").increment(1.0);

    tokio::spawn(async move {
        scopeguard::defer! {
            metrics::gauge!("active_connections").decrement(1.0);
        }

        let tx_clone = tx.clone();
        let batch = async move {
            // ── Phase 1: bind the live queue BEFORE replaying history. ──
            //
            // Every publisher inserts into `message_records` FIRST and
            // publishes to RabbitMQ second (see `message_insert_and_transmit`
            // and friends). Therefore, for any message M:
            //
            //   M published before the bind => M's DB insert also happened
            //     before the bind, and Phase 2 (which runs after the bind)
            //     replays it from the database;
            //   M published after the bind  => the queue is already bound,
            //     the broker retains M, and Phase 3 consumes it.
            //
            // No interleaving can lose M. The price is a possible duplicate
            // when M is both replayed and queued (at-least-once); clients
            // dedup by msg id.
            //
            // The original order (replay first, bind second) silently lost
            // every M published between the replay snapshot and the bind:
            // too late for the snapshot, no bound queue for the broker.
            let channel = connection
                .create_channel()
                .await
                .context("cannot create channel")?;
            // Per-stream queue (see `generate_stream_queue_name`): concurrent
            // streams for the same user each get their own queue, so the
            // declaration can never clash with a queue still held by another
            // pooled connection (the old user-named exclusive queue 405'd).
            // exclusive keeps the queue transient (RabbitMQ 4.x forbids
            // transient non-exclusive queues) and deletes it when the
            // declaring connection closes; auto_delete removes it as soon as
            // this stream's consumer goes away — whichever comes first.
            let queue_name = crate::rabbitmq::generate_stream_queue_name(id);
            tracing::info!("queue name: {}", queue_name);
            channel
                .queue_declare(
                    deadpool_lapin::lapin::types::ShortString::from(queue_name.clone()),
                    QueueDeclareOptions {
                        exclusive: true,
                        auto_delete: true,
                        durable: false,
                        ..Default::default()
                    },
                    FieldTable::default(),
                )
                .await
                .context("failed to create queue")?;
            create_user_message_direct_exchange(&channel).await?;
            channel
                .queue_bind(
                    deadpool_lapin::lapin::types::ShortString::from(queue_name.clone()),
                    deadpool_lapin::lapin::types::ShortString::from(
                        crate::rabbitmq::USER_MSG_DIRECT_EXCHANGE,
                    ),
                    deadpool_lapin::lapin::types::ShortString::from(
                        crate::rabbitmq::generate_route_key(id),
                    ),
                    QueueBindOptions::default(),
                    FieldTable::default(),
                )
                .await
                .context("failed to bind queue")?;
            create_user_message_broadcast_exchange(&channel).await?;
            channel
                .queue_bind(
                    deadpool_lapin::lapin::types::ShortString::from(queue_name.clone()),
                    deadpool_lapin::lapin::types::ShortString::from(
                        crate::rabbitmq::USER_MSG_BROADCAST_EXCHANGE,
                    ),
                    deadpool_lapin::lapin::types::ShortString::from(""),
                    QueueBindOptions::default(),
                    FieldTable::default(),
                )
                .await
                .context("failed to bind queue")?;
            tracing::trace!("starting to consume");
            // Registering the consumer already buffers deliveries inside
            // lapin; they are drained after the history replay below.
            let mut consumer = channel
                .basic_consume(
                    deadpool_lapin::lapin::types::ShortString::from(queue_name),
                    deadpool_lapin::lapin::types::ShortString::from(""),
                    deadpool_lapin::lapin::options::BasicConsumeOptions::default(),
                    FieldTable::default(),
                )
                .await
                .context("failed to consume")?;

            // ── Phase 2: replay history from the database. ──
            match db::messages::get_session_msgs(
                id,
                time.into(),
                &db_conn.db_pool,
                fetch_page_size,
                announcement_only,
            )
            .await
            {
                Ok(mut pag) => {
                    let mut sent_count: u64 = 0;
                    let db_logic = async {
                        while let Some(msgs) = pag.fetch_and_next().await? {
                            for msg_model in msgs {
                                let msg: RespondEventType =
                                    match serde_json::from_value(msg_model.msg_data) {
                                        Ok(m) => m,
                                        Err(e) => {
                                            tracing::warn!("incorrect msg in database:{e}");
                                            continue;
                                        }
                                    };
                                tx.send(Ok(FetchMsgsResponse {
                                    respond_event_type: Some(msg),
                                    msg_id: msg_model.msg_id as u64,
                                    time: Some(msg_model.time.into()),
                                }))
                                .await?;
                                sent_count += 1;
                                if history_limit > 0 && sent_count >= history_limit {
                                    return anyhow::Ok(());
                                }
                            }
                        }
                        anyhow::Ok(())
                    };
                    match db_logic.await {
                        Ok(_) => {}
                        Err(e) => {
                            tracing::error!("Database error:{e}");
                            tx.send(Err(Status::internal("Database error"))).await?;
                        }
                    }
                    anyhow::Ok(())
                }
                Err(e) => {
                    tx.send(Err(Status::internal("Unknown error"))).await?;
                    Err(e)?
                }
            }?;
            drop(db_conn);

            // ── Phase 3: drain the live consumer. ──
            tracing::trace!("starting consumer");
            let fetch = async {
                while let Some(delivery) = consumer.next().await {
                    tracing::trace!("deliver by rabbitmq");
                    let delivery = delivery?;
                    let msg = match FetchMsgsResponse::decode(delivery.data.as_slice()) {
                        Ok(m) => m,
                        Err(e) => {
                            tracing::warn!("incorrect msg in rabbitmq:{e}");
                            continue;
                        }
                    };
                    tx.send(Ok(msg)).await?;
                }
                anyhow::Ok(())
            };
            let check_connection = async {
                loop {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    if tx.is_closed() {
                        break;
                    }
                }
            };
            select! {
                err = fetch => {err?}
                _ = check_connection => {}
            }
            anyhow::Ok(())
        };
        match batch.await {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("Error occurred when listening to rabbitmq:{e}");
                match tx_clone.send(Err(Status::internal(SERVER_ERROR))).await {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("failed to send error:{e}");
                    }
                }
            }
        }
    });
    let output_stream = ReceiverStream::new(rx);
    Ok(Response::new(Box::pin(output_stream) as FetchMsgsStream))
}
