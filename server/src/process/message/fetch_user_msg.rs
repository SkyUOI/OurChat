use super::super::get_id_from_req;
use crate::{
    db,
    server::{FetchMsgsStream, RpcServer},
};
use anyhow::Context;
use base::time::from_google_timestamp;
use deadpool_lapin::lapin::options::{QueueBindOptions, QueueDeclareOptions, QueueDeleteOptions};
use deadpool_lapin::lapin::types::FieldTable;
use pb::ourchat::msg_delivery::v1::{
    FetchMsgsRequest, FetchMsgsResponse, Msg, fetch_msgs_response,
};
use prost::Message;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Response, Status};

pub async fn fetch_user_msg(
    server: &RpcServer,
    request: tonic::Request<FetchMsgsRequest>,
) -> Result<Response<FetchMsgsStream>, Status> {
    match fetch_user_msg_impl(server, request).await {
        Ok(d) => Ok(d),
        Err(e) => match e {
            FetchMsgError::Db(_) | FetchMsgError::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal("Server Error"))
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
    request: tonic::Request<FetchMsgsRequest>,
) -> Result<Response<FetchMsgsStream>, FetchMsgError> {
    let id = get_id_from_req(&request).unwrap();
    let request = request.into_inner();
    let time = match from_google_timestamp(&match request.time {
        Some(t) => t,
        None => {
            return Err(Status::invalid_argument("time is missing"))?;
        }
    }) {
        Some(t) => t,
        None => {
            return Err(Status::invalid_argument("time error"))?;
        }
    };
    let (tx, rx) = mpsc::channel(32);
    let db_conn = server.db.clone();
    let fetch_page_size = server.shared_data.cfg.main_cfg.db.fetch_msg_page_size;
    // add to rabbitmq
    let connection = server
        .rabbitmq
        .get()
        .await
        .context("cannot get rabbit connection")?;
    let channel = connection
        .create_channel()
        .await
        .context("cannot create channel")?;
    let queue_name = crate::rabbitmq::generate_client_name(id);
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

    tokio::spawn(async move {
        let send_to_client = async |res| match tx.send(res).await {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("send msg error:{e}");
            }
        };
        match db::messages::get_session_msgs(id, time.into(), &db_conn.db_pool, fetch_page_size)
            .await
        {
            Ok(mut pag) => {
                let db_logic = async {
                    while let Some(msgs) = pag.fetch_and_next().await? {
                        for msg in msgs {
                            let msg = match Msg::try_from(msg) {
                                Ok(m) => m,
                                Err(e) => {
                                    tracing::warn!("incorrect msg in database:{e}");
                                    continue;
                                }
                            };
                            send_to_client(Ok(FetchMsgsResponse {
                                data: Some(fetch_msgs_response::Data::Msg(msg)),
                            }))
                            .await;
                        }
                    }
                    Ok::<(), sea_orm::DbErr>(())
                };
                match db_logic.await {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("Database error:{e}");
                        send_to_client(Err(Status::internal("Database error"))).await;
                    }
                }
            }
            Err(e) => {
                tracing::error!("Database error:{e}");
                send_to_client(Err(Status::internal("Unknown error"))).await;
            }
        }
        // keep listening the rabbitmq
        let batch = async move {
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
                        .queue_delete(
                            &queue_name,
                            QueueDeleteOptions::default(),
                        )
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
            while let Some(delivery) = consumer.next().await {
                let delivery = delivery?;
                let msg = match Msg::decode(delivery.data.as_slice()) {
                    Ok(m) => m,
                    Err(e) => {
                        tracing::warn!("incorrect msg in rabbitmq:{e}");
                        continue;
                    }
                };
                send_to_client(Ok(FetchMsgsResponse {
                    data: Some(fetch_msgs_response::Data::Msg(msg)),
                }))
                .await;
            }
            anyhow::Ok(())
        };
        match batch.await {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("Error occurred when listening to rabbitmq:{e}");
                send_to_client(Err(Status::internal("Server Error"))).await;
            }
        }
    });
    let output_stream = ReceiverStream::new(rx);
    Ok(Response::new(Box::pin(output_stream) as FetchMsgsStream))
}
