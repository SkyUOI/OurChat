use anyhow::{Context, anyhow};
use base::consts::ID;
use chrono::Utc;
use migration::m20241229_022701_add_role_for_session::PredefinedPermissions;
use pb::service::ourchat::{
    msg_delivery::v1::fetch_msgs_response::RespondEventType,
    session::{
        e2eeize_and_dee2eeize_session::v1::{
            Dee2eeizeSessionRequest, Dee2eeizeSessionResponse, E2eeizeSessionRequest,
            E2eeizeSessionResponse,
        },
        session_room_key::v1::{SendRoomKeyNotification, UpdateRoomKeyNotification},
    },
};
use sea_orm::ActiveModelTrait;
use sea_orm::{ActiveValue, IntoActiveModel};
use tonic::{Request, Response, Status};

use crate::{
    db::{
        session::{get_members, get_session_by_id, if_permission_exist, in_session},
        user::get_account_info_db,
    },
    process::{
        Dest, MsgInsTransmitErr,
        error_msg::{PERMISSION_DENIED, SERVER_ERROR, not_found},
        message_insert_and_transmit,
    },
    server::RpcServer,
};

pub async fn e2eeize_session(
    server: &RpcServer,
    id: ID,
    request: Request<E2eeizeSessionRequest>,
) -> Result<Response<E2eeizeSessionResponse>, Status> {
    match e2eeize_session_impl(server, id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => {
            let status = match e {
                E2eeizeSessionError::Db(_)
                | E2eeizeSessionError::Unknown(_)
                | E2eeizeSessionError::MessageError(_) => {
                    tracing::error!("{}", e);
                    Status::internal(SERVER_ERROR)
                }
                E2eeizeSessionError::Status(s) => s,
            };
            Err(status)
        }
    }
}

#[derive(thiserror::Error, Debug)]
enum E2eeizeSessionError {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("unknown error:{0:?}")]
    Unknown(#[from] anyhow::Error),
    #[error("message error:{0:?}")]
    MessageError(#[from] MsgInsTransmitErr),
}

async fn e2eeize_session_impl(
    server: &RpcServer,
    id: ID,
    request: Request<E2eeizeSessionRequest>,
) -> Result<E2eeizeSessionResponse, E2eeizeSessionError> {
    let session_id = request.into_inner().session_id;
    let session = get_session_by_id(session_id.into(), &server.db.db_pool)
        .await?
        .ok_or(anyhow!("cannot find session"))?;
    if !in_session(id, session_id.into(), &server.db.db_pool).await? {
        Err(Status::not_found(not_found::USER_IN_SESSION))?;
    }
    if !if_permission_exist(
        id,
        session_id.into(),
        PredefinedPermissions::E2eeizeAndDee2eeizeSession.into(),
        &server.db.db_pool,
    )
    .await?
    {
        return Err(E2eeizeSessionError::Status(Status::permission_denied(
            PERMISSION_DENIED,
        )));
    }
    if session.e2ee_on {
        Err(Status::already_exists("session already e2eeized"))?;
    }
    let msg = RespondEventType::UpdateRoomKey(UpdateRoomKeyNotification { session_id });
    let rmq_conn = server
        .rabbitmq
        .get()
        .await
        .context("cannot get rabbit connection")?;
    let mut conn = rmq_conn
        .create_channel()
        .await
        .context("cannot create rabbitmq channel")?;
    message_insert_and_transmit(
        None,
        Some(session_id.into()),
        msg,
        Dest::User(id),
        false,
        &server.db.db_pool,
        &mut conn,
    )
    .await?;
    let sender_id: u64 = id.into();
    for members in get_members(session_id.into(), &server.db.db_pool).await? {
        if members.user_id != sender_id as i64 {
            let user = get_account_info_db(members.user_id.into(), &server.db.db_pool)
                .await?
                .ok_or(anyhow!("cannot find user"))?;
            let msg = RespondEventType::SendRoomKey(SendRoomKeyNotification {
                session_id,
                sender: (members.user_id as u64),
                public_key: user.public_key.into(),
            });
            message_insert_and_transmit(
                ID::from(members.user_id).into(),
                Some(session_id.into()),
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
    Ok(E2eeizeSessionResponse {})
}
pub async fn dee2eeize_session(
    server: &RpcServer,
    id: ID,
    request: Request<Dee2eeizeSessionRequest>,
) -> Result<Response<Dee2eeizeSessionResponse>, Status> {
    match dee2eeize_session_impl(server, id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => {
            let status = match e {
                Dee2eeizeSessionError::Db(_)
                | Dee2eeizeSessionError::Unknown(_)
                | Dee2eeizeSessionError::MessageError(_) => {
                    tracing::error!("{}", e);
                    Status::internal(SERVER_ERROR)
                }
                Dee2eeizeSessionError::Status(s) => s,
            };
            Err(status)
        }
    }
}

#[derive(thiserror::Error, Debug)]
enum Dee2eeizeSessionError {
    #[error("status error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Unknown(#[from] anyhow::Error),
    #[error("message error:{0:?}")]
    MessageError(#[from] MsgInsTransmitErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
}

async fn dee2eeize_session_impl(
    server: &RpcServer,
    id: ID,
    request: Request<Dee2eeizeSessionRequest>,
) -> Result<Dee2eeizeSessionResponse, Dee2eeizeSessionError> {
    let session_id = request.into_inner().session_id;
    let session = get_session_by_id(session_id.into(), &server.db.db_pool)
        .await?
        .ok_or(anyhow!("cannot find session"))?;
    if !in_session(id, session_id.into(), &server.db.db_pool).await? {
        Err(Status::not_found(not_found::USER_IN_SESSION))?;
    }
    if !if_permission_exist(
        id,
        session_id.into(),
        PredefinedPermissions::E2eeizeAndDee2eeizeSession.into(),
        &server.db.db_pool,
    )
    .await?
    {
        return Err(Dee2eeizeSessionError::Status(Status::permission_denied(
            PERMISSION_DENIED,
        )));
    }
    if !session.e2ee_on {
        Err(Status::already_exists("session already dee2eeized"))?;
    }
    let mut session = session.into_active_model();
    session.e2ee_on = ActiveValue::Set(false);
    session.update(&server.db.db_pool).await?;
    Ok(Dee2eeizeSessionResponse {})
}
