use super::get_id_from_req;
use crate::{
    component::EmailSender,
    consts::{ID, MsgID},
    entities::user_chat_msg,
    pb::msg_delivery::{SendMsgRequest, SendMsgResponse},
    server::{RpcServer, SendMsgStream},
};
use futures_util::StreamExt;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};

#[derive(Debug, thiserror::Error)]
enum SendMsgError {
    #[error("database error:{0}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("unknown error:{0}")]
    UnknownError(#[from] anyhow::Error),
    #[error("status:{0}")]
    StatusError(#[from] tonic::Status),
}

pub async fn send_msg(
    server: &RpcServer<impl EmailSender>,
    request: Request<Streaming<SendMsgRequest>>,
) -> Result<Response<SendMsgStream>, Status> {
    let id = get_id_from_req(&request).unwrap();
    let mut req = request.into_inner();
    let (tx, rx) = mpsc::channel(32);
    let db_conn = server.db.clone();
    tokio::spawn(async move {
        while let Some(msg) = req.next().await {
            let request = match msg {
                Ok(m) => m,
                Err(e) => {
                    tracing::error!("send msg error:{e}");
                    tx.send(Err(e)).await.ok();
                    continue;
                }
            };
            // TODO:store in binary data
            match insert_msg_record(
                id,
                ID(request.session_id),
                serde_json::value::to_value(request.bundle_msg).unwrap(),
                &db_conn.db_pool,
            )
            .await
            {
                Ok(msg_id) => {
                    tx.send(Ok(SendMsgResponse {
                        msg_id: msg_id.into(),
                    }))
                    .await
                    .ok();
                }
                Err(SendMsgError::DbError(e)) => {
                    tracing::error!("Database error:{e}");
                    tx.send(Err(Status::internal("database error"))).await.ok();
                }
                Err(SendMsgError::UnknownError(e)) => {
                    tracing::error!("Unknown error:{e}");
                    tx.send(Err(Status::internal("unknown error"))).await.ok();
                }
                Err(SendMsgError::StatusError(e)) => {
                    tx.send(Err(e)).await.ok();
                }
            };
        }
    });
    let output_stream = ReceiverStream::new(rx);
    Ok(Response::new(Box::pin(output_stream) as SendMsgStream))
}

async fn insert_msg_record(
    user_id: ID,
    session_id: ID,
    msg: serde_json::Value,
    db_conn: &DatabaseConnection,
) -> Result<MsgID, SendMsgError> {
    let msg = user_chat_msg::ActiveModel {
        msg_data: ActiveValue::Set(msg),
        sender_id: ActiveValue::Set(user_id.into()),
        session_id: ActiveValue::Set(session_id.into()),
        ..Default::default()
    };
    let msg = msg.insert(db_conn).await?;
    Ok(msg.chat_msg_id.into())
}
