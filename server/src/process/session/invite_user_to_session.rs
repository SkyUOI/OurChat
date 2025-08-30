use base::consts::ID;
use pb::service::ourchat::session::{
    invite_user_to_session::v1::{InviteUserToSessionRequest, InviteUserToSessionResponse},
    new_session::v1::{FailedMember, FailedReason},
};
use tonic::{Request, Response, Status};
use tracing::error;

use crate::{
    db::session::get_session_by_id,
    process::{
        check_user_exist,
        error_msg::{SERVER_ERROR, not_found},
        session::new_session::send_verification_request,
    },
    server::RpcServer,
};

#[derive(Debug, thiserror::Error)]
pub enum InviteToSessionError {
    #[error("unknown: {0:?}")]
    Unknown(#[from] anyhow::Error),
    #[error("database error: {0:?}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("status error: {0:?}")]
    Status(#[from] Status),
}

async fn invite_user_to_session_impl(
    server: &RpcServer,
    id: ID,
    request: Request<InviteUserToSessionRequest>,
) -> Result<InviteUserToSessionResponse, InviteToSessionError> {
    let req = request.into_inner();
    if get_session_by_id(req.session_id.into(), &server.db.db_pool)
        .await?
        .is_none()
    {
        return Err(InviteToSessionError::Status(Status::not_found(
            not_found::SESSION,
        )));
    }
    let mut failed_member = None;
    if !check_user_exist(req.invitee.into(), &server.db.db_pool).await? {
        failed_member = Some(FailedMember {
            id: req.invitee,
            reason: FailedReason::MemberNotFound.into(),
        });
    }
    if failed_member.is_none() {
        send_verification_request(
            server,
            id,
            req.invitee.into(),
            req.session_id.into(),
            req.leave_message,
        )
        .await?;
    }
    Ok(InviteUserToSessionResponse { failed_member })
}

pub async fn invite_user_to_session(
    server: &RpcServer,
    id: ID,
    request: Request<InviteUserToSessionRequest>,
) -> Result<Response<InviteUserToSessionResponse>, Status> {
    match invite_user_to_session_impl(server, id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            InviteToSessionError::Unknown(_) | InviteToSessionError::DbError(_) => {
                error!("{e}");
                Err(Status::internal(SERVER_ERROR))
            }
            InviteToSessionError::Status(status) => Err(status),
        },
    }
}
