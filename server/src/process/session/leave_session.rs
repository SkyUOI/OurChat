use crate::db::session::check_user_in_session;
use crate::process::error_msg::not_found;
use crate::process::get_id_from_req;
use crate::{db, process::error_msg::SERVER_ERROR, server::RpcServer};
use base::consts::SessionID;
use pb::service::ourchat::session::leave_session::v1::{LeaveSessionRequest, LeaveSessionResponse};
use sea_orm::TransactionTrait;
use tonic::{Request, Response, Status};

pub async fn leave_session(
    server: &RpcServer,
    request: Request<LeaveSessionRequest>,
) -> Result<Response<LeaveSessionResponse>, Status> {
    match leave_session_impl(server, request).await {
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
    request: Request<LeaveSessionRequest>,
) -> Result<LeaveSessionResponse, LeaveSessionErr> {
    let id = get_id_from_req(&request).unwrap();
    let req = request.into_inner();
    let session_id: SessionID = req.session_id.into();
    if check_user_in_session(id, session_id, &server.db.db_pool).await? {
        Err(Status::not_found(not_found::USER_IN_SESSION))?;
    }
    let transaction = server.db.db_pool.begin().await?;
    db::session::leave_session(session_id, id, &transaction).await?;
    transaction.commit().await?;
    tracing::debug!("User {} leave session {}", id, session_id);
    let ret = LeaveSessionResponse {};
    Ok(ret)
}
