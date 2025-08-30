use crate::db::session::{get_members, get_session_by_id, user_muted_status};
use crate::db::user::get_account_info_db;
use crate::process::{Dest, MsgInsTransmitErr, error_msg, message_insert_and_transmit};
use crate::{
    db::{messages::MsgError, session::in_session},
    process::error_msg::{PERMISSION_DENIED, SERVER_ERROR, not_found},
    server::RpcServer,
};
use anyhow::{Context, anyhow};
use base::consts::ID;
use chrono::Utc;
use pb::service::ourchat::msg_delivery::v1::fetch_msgs_response::RespondEventType;
use pb::service::ourchat::msg_delivery::v1::{Msg, SendMsgRequest, SendMsgResponse};
use pb::service::ourchat::session::session_room_key::v1::{
    SendRoomKeyNotification, UpdateRoomKeyNotification,
};
use pb::time::to_google_timestamp;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue, IntoActiveModel};
use tonic::{Request, Response, Status};

pub async fn send_msg(
    server: &RpcServer,
    id: ID,
    request: Request<SendMsgRequest>,
) -> Result<Response<SendMsgResponse>, Status> {
    match send_msg_impl(server, id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => Err(match e {
            SendMsgErr::Db(_)
            | SendMsgErr::Internal(_)
            | SendMsgErr::Redis(_)
            | SendMsgErr::MessageError(_) => {
                tracing::error!("{}", e);
                Status::internal(SERVER_ERROR)
            }
            SendMsgErr::Status(status) => status,
        }),
    }
}

#[derive(thiserror::Error, Debug)]
enum SendMsgErr {
    #[error("database error:{0:?}")]
    Db(#[from] DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
    #[error("redis error:{0:?}")]
    Redis(#[from] deadpool_redis::redis::RedisError),
    #[error("message error:{0:?}")]
    MessageError(#[from] MsgInsTransmitErr),
}

impl From<MsgError> for SendMsgErr {
    fn from(e: MsgError) -> Self {
        match e {
            MsgError::DbError(e) => Self::Db(e),
            MsgError::UnknownError(error) => Self::Internal(error),
            MsgError::PermissionDenied => {
                Self::Status(Status::permission_denied(PERMISSION_DENIED))
            }
            MsgError::NotFound => Self::Status(Status::not_found(not_found::MSG)),
        }
    }
}

async fn send_msg_impl(
    server: &RpcServer,
    id: ID,
    request: Request<SendMsgRequest>,
) -> Result<SendMsgResponse, SendMsgErr> {
    let req = request.into_inner();
    let db_conn = server.db.clone();
    // check
    if !in_session(id, req.session_id.into(), &db_conn.db_pool).await? {
        Err(Status::permission_denied(not_found::USER_IN_SESSION))?;
    }
    // check mute
    let mut redis_connection = server
        .db
        .redis_pool
        .get()
        .await
        .context("cannot get redis connection")?;
    if user_muted_status(id, req.session_id.into(), &mut redis_connection)
        .await?
        .is_some()
    {
        Err(Status::permission_denied(error_msg::MUTE))?
    }
    let session_id = req.session_id.into();
    let session = get_session_by_id(session_id, &server.db.db_pool)
        .await?
        .ok_or(anyhow!("cannot find session"))?;
    if !session.e2ee_on && req.is_encrypted {
        Err(Status::permission_denied(error_msg::E2EE_NOT_ON))?
    }
    let respond_msg = RespondEventType::Msg(Msg {
        bundle_msgs: req.bundle_msgs,
        session_id: req.session_id,
        is_encrypted: req.is_encrypted,
        sender_id: id.into(),
    });

    let sender_id: u64 = id.into();
    let rmq_conn = server
        .rabbitmq
        .get()
        .await
        .context("cannot get rabbit connection")?;
    let mut conn = rmq_conn
        .create_channel()
        .await
        .context("cannot create rabbitmq channel")?;

    let msg_id = message_insert_and_transmit(
        Some(id),
        Some(req.session_id.into()),
        respond_msg,
        Dest::Session(session_id),
        req.is_encrypted,
        &db_conn.db_pool,
        &mut conn,
    )
    .await?;
    if session.e2ee_on {
        let last_time = session.room_key_time.with_timezone(&Utc);
        let expire_time: chrono::TimeDelta =
            chrono::Duration::from_std(server.shared_data.cfg.main_cfg.room_key_duration).unwrap();
        if Utc::now() - last_time > expire_time || session.leaving_to_process {
            let msg = RespondEventType::UpdateRoomKey(UpdateRoomKeyNotification {
                session_id: session_id.into(),
            });
            message_insert_and_transmit(
                None,
                Some(session_id),
                msg,
                Dest::User(id),
                false,
                &server.db.db_pool,
                &mut conn,
            )
            .await?;
            for members in get_members(session_id, &server.db.db_pool).await? {
                if members.user_id != sender_id as i64 {
                    let user = get_account_info_db(members.user_id.into(), &server.db.db_pool)
                        .await?
                        .ok_or(anyhow!("cannot find user"))?;
                    let msg = RespondEventType::SendRoomKey(SendRoomKeyNotification {
                        session_id: session_id.into(),
                        sender: members.user_id as u64,
                        public_key: user.public_key.into(),
                    });
                    message_insert_and_transmit(
                        ID::from(members.user_id).into(),
                        Some(session_id),
                        msg,
                        Dest::User(id),
                        false,
                        &server.db.db_pool,
                        &mut conn,
                    )
                    .await?;
                }
            }
            let mut session = session.into_active_model();
            session.room_key_time = ActiveValue::Set(Utc::now().into());
            session.leaving_to_process = ActiveValue::Set(false);
            session.update(&server.db.db_pool).await?;
        }
    }
    Ok(SendMsgResponse {
        msg_id: msg_id.msg_id as u64,
        time: Some(to_google_timestamp(msg_id.time.into())),
    })
}
