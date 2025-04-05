use crate::{
    db,
    process::error_msg::{SERVER_ERROR, TIME_FORMAT_ERROR, TIME_MISSING},
    server::{FetchMsgsStream, RpcServer},
};
use anyhow::{Context, bail};
use base::consts::ID;
use deadpool_lapin::lapin::options::{QueueBindOptions, QueueDeclareOptions, QueueDeleteOptions};
use deadpool_lapin::lapin::types::FieldTable;
use pb::service::ourchat::msg_delivery::v1::{
    FetchMsgsRequest, FetchMsgsResponse, fetch_msgs_response::RespondMsgType,
};
use pb::time::{from_google_timestamp, to_google_timestamp};
use prost::Message;
use tokio::sync::mpsc;
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
    let time = match from_google_timestamp(&match request.time {
        Some(t) => t,
        None => {
            return Err(Status::invalid_argument(TIME_MISSING))?;
        }
    }) {
        Some(t) => t,
        None => {
            return Err(Status::invalid_argument(TIME_FORMAT_ERROR))?;
        }
    };
    let (tx, rx) = mpsc::channel(32);
    let db_conn = server.db.clone();
    let fetch_page_size = server.shared_data.cfg.main_cfg.db.fetch_msg_page_size;
    let connection = server
        .rabbitmq
        .get()
        .await
        .context("cannot get rabbit connection")?;
    tokio::spawn(async move {
        let send_to_client = async |res| match tx.send(res).await {
            Ok(_) => Ok(()),
            Err(_) => {
                tracing::info!("send msg failed:channel to client was closed");
                bail!("channel to client was closed");
            }
        };
        match db::messages::get_session_msgs(id, time.into(), &db_conn.db_pool, fetch_page_size)
            .await
        {
            Ok(mut pag) => {
                let db_logic = async {
                    while let Some(msgs) = pag.fetch_and_next().await? {
                        for msg_model in msgs {
                            let msg: RespondMsgType =
                                match serde_json::from_value(msg_model.msg_data) {
                                    Ok(m) => m,
                                    Err(e) => {
                                        tracing::warn!("incorrect msg in database:{e}");
                                        continue;
                                    }
                                };
                            send_to_client(Ok(FetchMsgsResponse {
                                respond_msg_type: Some(msg),
                                msg_id: msg_model.msg_id as u64,
                                time: Some(to_google_timestamp(msg_model.time.into())),
                            }))
                            .await?;
                        }
                    }
                    anyhow::Ok(())
                };
                match db_logic.await {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("Database error:{e}");
                        send_to_client(Err(Status::internal("Database error"))).await?;
                    }
                }
                anyhow::Ok(())
            }
            Err(e) => {
                send_to_client(Err(Status::internal("Unknown error"))).await?;
                Err(e)?
            }
        }?;
        // keep listening the rabbitmq
        // add to rabbitmq
        let channel = connection
            .create_channel()
            .await
            .context("cannot create channel")?;
        let queue_name = crate::rabbitmq::generate_client_name(id);
        tracing::info!("queue name: {}", queue_name);
        channel
            .queue_declare(
                &queue_name,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await
            .context("failed to create queue")?;
        channel
            .queue_bind(
                &queue_name,
                crate::rabbitmq::USER_MSG_EXCHANGE,
                &crate::rabbitmq::generate_route_key(id),
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await
            .context("failed to bind queue")?;
        channel
            .queue_bind(
                &queue_name,
                crate::rabbitmq::USER_MSG_EXCHANGE,
                "",
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await
            .context("failed to bind queue")?;
        let batch = async move {
            tracing::trace!("starting to consume");
            let mut consumer = channel
                .basic_consume(
                    &queue_name,
                    "",
                    deadpool_lapin::lapin::options::BasicConsumeOptions::default(),
                    FieldTable::default(),
                )
                .await
                .context("failed to consume")?;
            let _channel = scopeguard::guard(channel, |channel| {
                tokio::spawn(async move {
                    match channel
                        .queue_delete(&queue_name, QueueDeleteOptions::default())
                        .await
                    {
                        Ok(_) => {
                            tracing::trace!("queue deleted");
                        }
                        Err(e) => {
                            tracing::error!("failed to delete queue:{e}");
                        }
                    }
                });
            });
            tracing::trace!("starting consumer");
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
                send_to_client(Ok(msg)).await?;
            }
            anyhow::Ok(())
        };
        match batch.await {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("Error occurred when listening to rabbitmq:{e}");
                send_to_client(Err(Status::internal(SERVER_ERROR))).await?;
            }
        }
        anyhow::Ok(())
    });
    let output_stream = ReceiverStream::new(rx);
    Ok(Response::new(Box::pin(output_stream) as FetchMsgsStream))
}
