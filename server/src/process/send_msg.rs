use super::get_id_from_req;
use crate::{
    component::EmailSender,
    consts::{ID, MsgID},
    entities::user_chat_msg,
    pb::ourchat::msg_delivery::v1::{SendMsgRequest, SendMsgResponse},
    server::RpcServer,
};
use futures_util::StreamExt;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};

#[derive(Debug, thiserror::Error)]
enum SendMsgError {
    #[error("database error:{0}")]
    Db(#[from] sea_orm::DbErr),
    #[error("unknown error:{0}")]
    Unknown(#[from] anyhow::Error),
    #[error("status:{0}")]
    Status(#[from] tonic::Status),
}

pub async fn send_msg(
    server: &RpcServer<impl EmailSender>,
    request: Request<SendMsgRequest>,
) -> Result<Response<SendMsgResponse>, Status> {
    let id = get_id_from_req(&request).unwrap();
    let req = request.into_inner();
    let db_conn = server.db.clone();
    // TODO:store in binary data
    match insert_msg_record(
        id,
        ID(req.session_id),
        serde_json::value::to_value(req.bundle_msgs).unwrap(),
        &db_conn.db_pool,
    )
    .await
    {
        Ok(msg_id) => Ok(Response::new(SendMsgResponse {
            msg_id: msg_id.into(),
        })),
        Err(SendMsgError::Db(e)) => {
            tracing::error!("Database error:{e}");
            Err(Status::internal("database error"))
        }
        Err(SendMsgError::Unknown(e)) => {
            tracing::error!("Unknown error:{e}");
            Err(Status::internal("unknown error"))
        }
        Err(SendMsgError::Status(e)) => Err(e),
    }
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
