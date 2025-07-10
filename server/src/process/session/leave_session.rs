use crate::db::session::{SessionError, check_user_in_session, get_session_by_id};
use crate::process::error_msg::not_found;
use crate::{db, process::error_msg::SERVER_ERROR, server::RpcServer};
use anyhow::anyhow;
use base::consts::{ID, SessionID};
use pb::service::ourchat::session::leave_session::v1::{LeaveSessionRequest, LeaveSessionResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, IntoActiveModel, TransactionTrait};
use tonic::{Request, Response, Status};

pub async fn leave_session(
    server: &RpcServer,
    id: ID,
    request: Request<LeaveSessionRequest>,
) -> Result<Response<LeaveSessionResponse>, Status> {
    match leave_session_impl(server, id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            LeaveSessionErr::Db(_) | LeaveSessionErr::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            LeaveSessionErr::Status(status) => Err(status),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum LeaveSessionErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
}

async fn leave_session_impl(
    server: &RpcServer,
    id: ID,
    request: Request<LeaveSessionRequest>,
) -> Result<LeaveSessionResponse, LeaveSessionErr> {
    let req = request.into_inner();
    let session_id: SessionID = req.session_id.into();
    if check_user_in_session(id, session_id, &server.db.db_pool).await? {
        Err(Status::not_found(not_found::USER_IN_SESSION))?;
    }
    let transaction = server.db.db_pool.begin().await?;
    let session = get_session_by_id(session_id, &server.db.db_pool)
        .await?
        .ok_or(anyhow!("cannot found session"))?;
    if session.e2ee_on {
        let mut session = session.into_active_model();
        session.leaving_to_process = ActiveValue::Set(true);
        session.update(&server.db.db_pool).await?;
    }
    match db::session::leave_session(session_id, id, &transaction).await {
        Ok(_) => {
            transaction.commit().await?;
        }
        Err(SessionError::SessionNotFound) => {
            tracing::error!("Relation exist but session not found");
            transaction.rollback().await?;
            Err(Status::not_found(not_found::SESSION))?
        }
        Err(SessionError::Db(e)) => {
            transaction.rollback().await?;
            return Err(LeaveSessionErr::Db(e));
        }
    }
    tracing::debug!("User {} leave session {}", id, session_id);
    let ret = LeaveSessionResponse {};
    Ok(ret)
}
