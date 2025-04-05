use crate::db::session::{SessionError, join_in_session, user_banned_status};
use crate::process::error_msg;
use crate::process::error_msg::not_found;
use crate::{process::error_msg::SERVER_ERROR, server::RpcServer};
use anyhow::Context;
use base::consts::{ID, SessionID};
use entities::message_records;
use pb::service::ourchat::session::accept_session::v1::{
    AcceptSessionRequest, AcceptSessionResponse,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use tonic::{Response, Status};

#[derive(Debug, thiserror::Error)]
enum AcceptSessionError {
    #[error("database error:{0:?}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("unknown error:{0:?}")]
    UnknownError(#[from] anyhow::Error),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("redis error:{0:?}")]
    Redis(#[from] deadpool_redis::redis::RedisError),
}

async fn accept_impl(
    server: &RpcServer,
    id: ID,
    request: tonic::Request<AcceptSessionRequest>,
) -> Result<AcceptSessionResponse, AcceptSessionError> {
    let req = request.into_inner();
    let session_id: SessionID = req.session_id.into();
    // check if banned from the session
    if user_banned_status(
        id,
        session_id,
        &mut server
            .db
            .redis_pool
            .get()
            .await
            .context("cannot get redis connection")?,
    )
    .await?
    .is_some()
    {
        Err(Status::permission_denied(error_msg::BAN))?;
    }
    // check if the invitation is valid
    let time_limit = chrono::Utc::now()
        - chrono::Duration::from_std(server.shared_data.cfg.main_cfg.verification_expire_time)
            .unwrap();
    let model = entities::message_records::Entity::find()
        .filter(message_records::Column::SessionId.eq(req.session_id))
        .filter(message_records::Column::SenderId.eq(id))
        .filter(message_records::Column::Time.gt(time_limit))
        .one(&server.db.db_pool)
        .await?;
    if model.is_none() {
        Err(Status::not_found(not_found::SESSION_INVITATION))?
    }
    if req.accepted {
        let transaction = server.db.db_pool.begin().await?;
        match join_in_session(session_id, id, None, &transaction).await {
            Ok(_) => {
                transaction.commit().await?;
            }
            Err(SessionError::Db(e)) => {
                transaction.rollback().await?;
                return Err(AcceptSessionError::DbError(e));
            }
            Err(SessionError::SessionNotFound) => {
                transaction.rollback().await?;
                return Err(AcceptSessionError::Status(Status::not_found(
                    not_found::SESSION,
                )));
            }
        }
    }
    Ok(AcceptSessionResponse {})
}

pub async fn accept_session(
    server: &RpcServer,
    id: ID,
    request: tonic::Request<AcceptSessionRequest>,
) -> Result<Response<AcceptSessionResponse>, Status> {
    match accept_impl(server, id, request).await {
        Ok(d) => Ok(Response::new(d)),
        Err(e) => match e {
            AcceptSessionError::DbError(_)
            | AcceptSessionError::UnknownError(_)
            | AcceptSessionError::Redis(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            AcceptSessionError::Status(s) => Err(s),
        },
    }
}
