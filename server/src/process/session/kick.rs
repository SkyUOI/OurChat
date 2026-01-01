use crate::db::manager::manage_permission_existed;
use crate::db::session::{SessionError, get_session_by_id, if_permission_exist, leave_session};
use crate::process::error_msg::{PERMISSION_DENIED, not_found};
use crate::{process::error_msg::SERVER_ERROR, server::RpcServer};
use base::consts::{ID, SessionID};
use migration::predefined::{PredefinedPermissions, PredefinedServerManagementPermission};
use pb::service::ourchat::session::kick::v1::{KickUserRequest, KickUserResponse};
use sea_orm::TransactionTrait;
use tonic::{Request, Response, Status};

pub async fn kick_user(
    server: &RpcServer,
    id: ID,
    request: Request<KickUserRequest>,
) -> Result<Response<KickUserResponse>, Status> {
    match kick_user_impl(server, id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            KickUserErr::Db(_) | KickUserErr::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            KickUserErr::Status(status) => Err(status),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum KickUserErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
}

async fn kick_user_impl(
    server: &RpcServer,
    id: ID,
    request: Request<KickUserRequest>,
) -> Result<KickUserResponse, KickUserErr> {
    let req = request.into_inner();
    let session_id: SessionID = req.session_id.into();
    let target_user_id: ID = req.user_id.into();

    // check if session exists
    if get_session_by_id(session_id, &server.db.db_pool)
        .await?
        .is_none()
    {
        Err(Status::not_found(not_found::SESSION))?;
    }

    // Check if user has ManageSessions server management permission
    // Admins with this permission can kick any user from any session
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
            PredefinedPermissions::KickUser.into(),
            &server.db.db_pool,
        )
        .await?
        {
            Err(Status::permission_denied(PERMISSION_DENIED))?
        }
    }

    // Remove the user from the session
    let transaction = server.db.db_pool.begin().await?;
    match leave_session(session_id, target_user_id, &transaction).await {
        Ok(_) => {
            transaction.commit().await?;
        }
        Err(SessionError::Db(e)) => {
            Err(e)?;
        }
        Err(SessionError::SessionNotFound) => {
            Err(Status::not_found(not_found::SESSION))?;
        }
    }

    Ok(KickUserResponse {})
}
