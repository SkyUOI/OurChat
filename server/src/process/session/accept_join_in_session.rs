use crate::db::session::{SessionError, if_permission_exist};
use crate::process::error_msg::{PERMISSION_DENIED, not_found};
use crate::process::{Dest, MsgInsTransmitErr};
use crate::{db, process::error_msg::SERVER_ERROR, server::RpcServer};
use anyhow::Context;
use base::consts::{ID, SessionID};
use migration::m20241229_022701_add_role_for_session::PredefinedPermissions;
use pb::service::ourchat::msg_delivery::v1::fetch_msgs_response::RespondMsgType;
use pb::service::ourchat::session::join_in_session::v1::{
    AcceptJoinInSessionNotification, AcceptJoinInSessionRequest, AcceptJoinInSessionResponse,
};
use sea_orm::TransactionTrait;
use tonic::{Request, Response, Status};

pub async fn accept_join_in_session(
    server: &RpcServer,
    id: ID,
    request: Request<AcceptJoinInSessionRequest>,
) -> Result<Response<AcceptJoinInSessionResponse>, Status> {
    match accept_join_in_session_impl(server, id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            AcceptJoinInSessionErr::Db(_) | AcceptJoinInSessionErr::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            AcceptJoinInSessionErr::Status(status) => Err(status),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum AcceptJoinInSessionErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
}

impl From<MsgInsTransmitErr> for AcceptJoinInSessionErr {
    fn from(value: MsgInsTransmitErr) -> Self {
        match value {
            MsgInsTransmitErr::Db(db_err) => Self::Db(db_err),
            MsgInsTransmitErr::Unknown(error) => Self::Internal(error),
            MsgInsTransmitErr::PermissionDenied => {
                Self::Status(Status::permission_denied(PERMISSION_DENIED))
            }
            MsgInsTransmitErr::NotFound => {
                tracing::error!(
                    "Insert a new message record into the database, but a not found was returned."
                );
                Self::Status(Status::not_found(not_found::MSG))
            }
        }
    }
}

async fn accept_join_in_session_impl(
    server: &RpcServer,
    id: ID,
    request: Request<AcceptJoinInSessionRequest>,
) -> Result<AcceptJoinInSessionResponse, AcceptJoinInSessionErr> {
    let req = request.into_inner();
    let session_id: SessionID = req.session_id.into();
    if !if_permission_exist(
        id,
        session_id,
        PredefinedPermissions::AcceptJoinRequest.into(),
        &server.db.db_pool,
    )
    .await?
    {
        Err(Status::permission_denied(PERMISSION_DENIED))?
    }
    if req.accepted {
        let transaction = server.db.db_pool.begin().await?;
        match db::session::join_in_session(session_id, req.user_id.into(), None, &transaction).await
        {
            Ok(_) => {
                transaction.commit().await?;
            }
            Err(e) => {
                transaction.rollback().await?;
                match e {
                    SessionError::Db(e) => {
                        Err(e)?;
                    }
                    SessionError::SessionNotFound => {
                        Err(Status::not_found(not_found::SESSION))?;
                    }
                }
            }
        }
    }
    // send a notification to applicant
    let respond_msg = RespondMsgType::AcceptJoinInSession(AcceptJoinInSessionNotification {
        session_id: session_id.into(),
        accepted: req.accepted,
    });
    let rmq_conn = server
        .rabbitmq
        .get()
        .await
        .context("cannot get rabbitmq connection")?;
    let mut conn = rmq_conn
        .create_channel()
        .await
        .context("cannot create rabbitmq channel")?;
    super::super::message_insert_and_transmit(
        req.user_id.into(),
        Some(session_id),
        respond_msg.clone(),
        Dest::User(req.user_id.into()),
        false,
        &server.db.db_pool,
        &mut conn,
    )
    .await?;
    let ret = AcceptJoinInSessionResponse {};
    Ok(ret)
}
