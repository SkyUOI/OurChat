use anyhow::Context;
use base::constants::ID;
use pb::service::ourchat::{
    msg_delivery::v1::fetch_msgs_response::RespondEventType,
    session::{
        new_session::v1::{FailedMember, FailedReason},
        session_room_key::v1::{
            ReceiveRoomKeyNotification, SendRoomKeyRequest, SendRoomKeyResponse,
        },
    },
};
use tonic::{Request, Response, Status};

use crate::{
    db::session::get_session_by_id,
    process::{
        Dest, MsgInsTransmitErr, check_user_exist,
        error_msg::{SERVER_ERROR, not_found},
        message_insert_and_transmit,
    },
    server::RpcServer,
};

#[derive(Debug, thiserror::Error)]
pub enum SendRoomKeyError {
    #[error("unknown error: {0:?}")]
    Unknown(#[from] anyhow::Error),
    #[error("database error: {0:?}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("message error:{0:?}")]
    MessageError(#[from] MsgInsTransmitErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
}

pub async fn send_room_key(
    server: &RpcServer,
    id: ID,
    request: Request<SendRoomKeyRequest>,
) -> Result<Response<SendRoomKeyResponse>, Status> {
    match send_room_key_impl(server, id, request).await {
        Ok(response) => Ok(Response::new(response)),
        Err(e) => match e {
            SendRoomKeyError::DbError(_)
            | SendRoomKeyError::Unknown(_)
            | SendRoomKeyError::MessageError(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            SendRoomKeyError::Status(status) => Err(status),
        },
    }
}

async fn send_room_key_impl(
    server: &RpcServer,
    id: ID,
    request: Request<SendRoomKeyRequest>,
) -> Result<SendRoomKeyResponse, SendRoomKeyError> {
    let req = request.into_inner();
    if get_session_by_id(req.session_id.into(), &server.db.db_pool)
        .await?
        .is_none()
    {
        return Err(SendRoomKeyError::Status(Status::not_found(
            not_found::SESSION,
        )));
    }
    let mut failed_member = None;
    if !check_user_exist(req.user_id.into(), &server.db.db_pool).await? {
        failed_member = Some(FailedMember {
            id: req.user_id,
            reason: FailedReason::MemberNotFound.into(),
        });
    }

    let rmq_conn = server.get_rabbitmq_manager().await?;
    let mut conn = rmq_conn
        .create_channel()
        .await
        .context("cannot create rabbitmq channel")?;
    let session_id = req.session_id;
    let room_key = req.room_key;
    let msg = RespondEventType::ReceiveRoomKey(ReceiveRoomKeyNotification {
        session_id,
        user_id: id.into(),
        room_key,
    });
    message_insert_and_transmit(
        id.into(),
        Some(session_id.into()),
        msg,
        Dest::User(req.user_id.into()),
        false,
        &server.db.db_pool,
        &mut conn,
    )
    .await?;
    Ok(SendRoomKeyResponse { failed_member })
}
