use crate::db::manager::manage_permission_existed;
use crate::db::session::{SessionError, get_session_by_id, if_permission_exist};
use crate::process::error_msg::{PERMISSION_DENIED, not_found};
use crate::{db, process::error_msg::SERVER_ERROR, server::RpcServer};
use base::constants::{ID, SessionID};
use migration::predefined::{PredefinedPermissions, PredefinedServerManagementPermission};
use pb::service::ourchat::session::delete_session::v1::{
    DeleteSessionRequest, DeleteSessionResponse,
};
use tonic::{Request, Response, Status};

pub async fn delete_session(
    server: &RpcServer,
    id: ID,
    request: Request<DeleteSessionRequest>,
) -> Result<Response<DeleteSessionResponse>, Status> {
    match delete_session_impl(server, id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            DeleteSessionErr::Db(_) | DeleteSessionErr::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            DeleteSessionErr::Status(status) => Err(status),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum DeleteSessionErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
}

async fn delete_session_impl(
    server: &RpcServer,
    id: ID,
    request: Request<DeleteSessionRequest>,
) -> Result<DeleteSessionResponse, DeleteSessionErr> {
    let req = request.into_inner();
    let session_id: SessionID = req.session_id.into();
    // check if session exists
    if get_session_by_id(session_id, &server.db.db_pool)
        .await?
        .is_none()
    {
        Err(Status::not_found(not_found::SESSION))?;
    }
    // Check if user has ManageSessions server management permission
    // Admins with this permission can delete any session
    let has_admin_permission = manage_permission_existed(
        id,
        PredefinedServerManagementPermission::ManageSessions as i64,
        &server.db.db_pool,
    )
    .await?;

    if !has_admin_permission {
        // Normal user - check session-level permission
        if !if_permission_exist(
            id,
            session_id,
            PredefinedPermissions::DeleteSession.into(),
            &server.db.db_pool,
        )
        .await?
        {
            Err(Status::permission_denied(PERMISSION_DENIED))?
        }
    }
    match db::session::delete_session(session_id, &server.db.db_pool).await {
        Ok(_) => {}
        Err(SessionError::Db(e)) => {
            Err(e)?;
        }
        Err(SessionError::SessionNotFound) => {
            Err(Status::not_found(not_found::SESSION))?;
        }
    }
    let ret = DeleteSessionResponse {};
    Ok(ret)
}
