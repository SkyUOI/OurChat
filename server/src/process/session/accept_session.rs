use crate::db::session::{join_in_session, user_banned_status};
use crate::process::error_msg::not_found;
use crate::process::{error_msg, get_id_from_req};
use crate::{process::error_msg::SERVER_ERROR, server::RpcServer};
use anyhow::Context;
use base::consts::SessionID;
use entities::user_chat_msg;
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
    request: tonic::Request<AcceptSessionRequest>,
) -> Result<AcceptSessionResponse, AcceptSessionError> {
    let id = get_id_from_req(&request).unwrap();
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
        - chrono::Duration::days(server.shared_data.cfg.main_cfg.verification_expire_days as i64);
    let model = entities::user_chat_msg::Entity::find()
        .filter(user_chat_msg::Column::SessionId.eq(req.session_id))
        .filter(user_chat_msg::Column::SenderId.eq(id))
        .filter(user_chat_msg::Column::Time.gt(time_limit))
        .one(&server.db.db_pool)
        .await?;
    if model.is_none() {
        Err(Status::not_found(not_found::SESSION_INVITATION))?
    }
    if req.accepted {
        let transaction = server.db.db_pool.begin().await?;
        join_in_session(session_id, id, None, &transaction).await?;
        transaction.commit().await?;
    }
    Ok(AcceptSessionResponse {})
}

pub async fn accept_session(
    server: &RpcServer,
    request: tonic::Request<AcceptSessionRequest>,
) -> Result<Response<AcceptSessionResponse>, Status> {
    match accept_impl(server, request).await {
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
