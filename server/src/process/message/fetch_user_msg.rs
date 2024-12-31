use super::super::get_id_from_req;
use crate::{
    component::EmailSender,
    db,
    server::{FetchMsgsStream, RpcServer},
};
use base::time::from_google_timestamp;
use pb::ourchat::msg_delivery::v1::{
    FetchMsgsRequest, FetchMsgsResponse, Msg, fetch_msgs_response,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Response, Status};

pub async fn fetch_user_msg<T: EmailSender>(
    server: &RpcServer<T>,
    request: tonic::Request<FetchMsgsRequest>,
) -> Result<Response<FetchMsgsStream>, Status> {
    let id = get_id_from_req(&request).unwrap();
    let request = request.into_inner();
    let time = match from_google_timestamp(&match request.time {
        Some(t) => t,
        None => {
            return Err(Status::invalid_argument("time is missing"));
        }
    }) {
        Some(t) => t,
        None => {
            return Err(Status::invalid_argument("time error"));
        }
    };
    let (tx, rx) = mpsc::channel(32);
    let db_conn = server.db.clone();
    let fetch_page_size = server.shared_data.cfg.main_cfg.db.fetch_msg_page_size;
    server.shared_data.connected_clients.insert(id, tx.clone());
    tokio::spawn(async move {
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
                            match tx
                                .send(Ok(FetchMsgsResponse {
                                    data: Some(fetch_msgs_response::Data::Msg(msg)),
                                }))
                                .await
                            {
                                Ok(_) => {}
                                Err(e) => {
                                    tracing::error!("send msg error:{e}");
                                }
                            }
                        }
                    }
                    Ok::<(), sea_orm::DbErr>(())
                };
                match db_logic.await {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("Database error:{e}");
                        tx.send(Err(Status::internal("Database error"))).await.ok();
                    }
                }
            }
            Err(e) => {
                tracing::error!("Database error:{e}");
                tx.send(Err(Status::internal("Unknown error"))).await.ok();
            }
        }
    });
    let output_stream = ReceiverStream::new(rx);
    Ok(Response::new(Box::pin(output_stream) as FetchMsgsStream))
}
