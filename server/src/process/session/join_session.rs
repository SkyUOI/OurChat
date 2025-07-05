use crate::db::messages::insert_msg_record;
use crate::db::session::{
    get_all_session_relations, get_session_by_id, if_permission_exist, user_banned_status,
};
use crate::process::error_msg::{BAN, PERMISSION_DENIED, not_found};
use crate::process::{Dest, transmit_msg};
use crate::{db, process::error_msg::SERVER_ERROR, server::RpcServer};
use anyhow::Context;
use base::consts::{ID, SessionID};
use migration::m20241229_022701_add_role_for_session::PredefinedPermissions;
use pb::service::ourchat::msg_delivery::v1::FetchMsgsResponse;
use pb::service::ourchat::msg_delivery::v1::fetch_msgs_response::RespondEventType;
use pb::service::ourchat::session::join_session::v1::{
    JoinSessionApproval, JoinSessionRequest, JoinSessionResponse,
};
use pb::time::to_google_timestamp;
use tonic::{Request, Response, Status};

pub async fn join_session(
    server: &RpcServer,
    id: ID,
    request: Request<JoinSessionRequest>,
) -> Result<Response<JoinSessionResponse>, Status> {
    match join_session_impl(server, id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            JoinInSessionErr::Db(_)
            | JoinInSessionErr::Internal(_)
            | JoinInSessionErr::Redis(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            JoinInSessionErr::Status(status) => Err(status),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum JoinInSessionErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
    #[error("redis error:{0:?}")]
    Redis(#[from] deadpool_redis::redis::RedisError),
}

impl From<db::messages::MsgError> for JoinInSessionErr {
    fn from(value: db::messages::MsgError) -> Self {
        match value {
            db::messages::MsgError::DbError(db_err) => Self::Db(db_err),
            db::messages::MsgError::PermissionDenied => {
                Self::Status(Status::permission_denied(PERMISSION_DENIED))
            }
            db::messages::MsgError::NotFound => {
                tracing::error!(
                    "Insert a new message record into the database, but a not found was returned."
                );
                Self::Status(Status::not_found(not_found::MSG))
            }
            db::messages::MsgError::UnknownError(error) => Self::Internal(error),
        }
    }
}

async fn join_session_impl(
    server: &RpcServer,
    id: ID,
    request: Request<JoinSessionRequest>,
) -> Result<JoinSessionResponse, JoinInSessionErr> {
    let req = request.into_inner();
    let session_id: SessionID = req.session_id.into();

    // Check if session exists
    if get_session_by_id(session_id, &server.db.db_pool)
        .await?
        .is_none()
    {
        return Err(JoinInSessionErr::Status(Status::not_found(
            not_found::SESSION,
        )));
    }

    // Check if user is banned
    let mut conn = server
        .db
        .redis_pool
        .get()
        .await
        .context("cannot get redis connection")?;
    if user_banned_status(id, session_id, &mut conn)
        .await?
        .is_some()
    {
        return Err(JoinInSessionErr::Status(Status::permission_denied(BAN)));
    }

    let respond_msg = RespondEventType::JoinSessionApproval(JoinSessionApproval {
        session_id: session_id.into(),
        user_id: id.into(),
        leave_message: req.leave_message,
    });
    // TODO: is_encrypted
    let msg_model = insert_msg_record(
        id,
        Some(session_id),
        respond_msg.clone(),
        false,
        &server.db.db_pool,
        false,
    )
    .await?;
    // try to send the message directly
    let fetch_response = FetchMsgsResponse {
        msg_id: msg_model.msg_id as u64,
        time: Some(to_google_timestamp(msg_model.time.into())),
        respond_event_type: Some(respond_msg),
    };
    let peoples_should_be_sent = get_all_session_relations(id, &server.db.db_pool).await?;
    let rmq_conn = server
        .rabbitmq
        .get()
        .await
        .context("cannot get rabbit connection")?;
    let mut rmq_channel = rmq_conn
        .create_channel()
        .await
        .context("cannot create channel")?;
    for i in peoples_should_be_sent {
        if !if_permission_exist(
            i.user_id.into(),
            session_id,
            PredefinedPermissions::AcceptJoinRequest.into(),
            &server.db.db_pool,
        )
        .await?
        {
            continue;
        }
        transmit_msg(
            fetch_response.clone(),
            Dest::User(i.user_id.into()),
            &mut rmq_channel,
            &server.db.db_pool,
        )
        .await?;
    }
    let ret = JoinSessionResponse {};
    Ok(ret)
}
