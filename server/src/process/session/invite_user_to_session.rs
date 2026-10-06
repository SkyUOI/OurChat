use base::constants::ID;
use migration::predefined::SessionInvitationPolicy;
use pb::service::ourchat::session::{
    invite_user_to_session::v1::{InviteUserToSessionRequest, InviteUserToSessionResponse},
    new_session::v1::{FailedMember, FailedReason},
};
use tonic::{Request, Response, Status};
use tracing::error;

use crate::{
    db::session::get_session_by_id,
    db::user::get_account_info_db,
    process::{
        error_msg::{
            SERVER_ERROR, SESSION_INVITATION_FRIENDS_ONLY, SESSION_INVITATION_NOT_ALLOWED,
            not_found,
        },
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

/// Check whether the inviter satisfies the invitee's session invitation policy
async fn check_session_invitation_policy(
    server: &RpcServer,
    inviter: ID,
    invitee: ID,
    policy: i32,
) -> Result<(), InviteToSessionError> {
    let policy =
        SessionInvitationPolicy::try_from(policy).unwrap_or(SessionInvitationPolicy::AllowAll);
    match policy {
        SessionInvitationPolicy::AllowAll => Ok(()),
        SessionInvitationPolicy::FriendsOnly => {
            // friend relations are stored in both directions
            let is_friend = crate::db::friend::query_friend(inviter, invitee, &server.db.db_pool)
                .await?
                .is_some();
            if is_friend {
                Ok(())
            } else {
                Err(InviteToSessionError::Status(Status::permission_denied(
                    SESSION_INVITATION_FRIENDS_ONLY,
                )))
            }
        }
        SessionInvitationPolicy::Nobody => Err(InviteToSessionError::Status(
            Status::permission_denied(SESSION_INVITATION_NOT_ALLOWED),
        )),
    }
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
    match get_account_info_db(req.invitee.into(), &server.db.db_pool).await? {
        Some(invitee) => {
            // respect the invitee's session invitation policy
            check_session_invitation_policy(
                server,
                id,
                req.invitee.into(),
                invitee.session_invitation_policy,
            )
            .await?;
            send_verification_request(
                server,
                id,
                req.invitee.into(),
                req.session_id.into(),
                req.leave_message,
            )
            .await?;
        }
        None => {
            failed_member = Some(FailedMember {
                id: req.invitee,
                reason: FailedReason::MemberNotFound.into(),
            });
        }
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
