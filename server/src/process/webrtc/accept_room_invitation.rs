use crate::{
    process::error_msg::{SERVER_ERROR, not_found},
    server::RpcServer,
    webrtc::{RoomId, room_invitations_key, room_key, room_members_key},
};
use base::constants::ID;
use pb::service::ourchat::webrtc::room::accept_room_invitation::v1::{
    AcceptRoomInvitationRequest, AcceptRoomInvitationResponse,
};
use redis::AsyncTypedCommands;
use tonic::{Request, Response, Status};

pub async fn accept_room_invitation(
    server: &RpcServer,
    user_id: ID,
    request: Request<AcceptRoomInvitationRequest>,
) -> Result<Response<AcceptRoomInvitationResponse>, Status> {
    match accept_room_invitation_impl(server, user_id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            AcceptInviteErr::Redis(_) | AcceptInviteErr::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            AcceptInviteErr::RoomNotFound => Err(Status::not_found(not_found::WEBRTC_ROOM)),
            AcceptInviteErr::NotInvited => Err(Status::permission_denied(
                "you are not invited to this room",
            )),
            AcceptInviteErr::AlreadyInRoom => {
                Err(Status::already_exists("you are already in this room"))
            }
            AcceptInviteErr::Status(s) => Err(s),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum AcceptInviteErr {
    #[error("redis error:{0:?}")]
    Redis(#[from] redis::RedisError),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
    #[error("room not found")]
    RoomNotFound,
    #[error("not invited")]
    NotInvited,
    #[error("already in room")]
    AlreadyInRoom,
    #[error("status error:{0:?}")]
    Status(#[from] Status),
}

async fn accept_room_invitation_impl(
    server: &RpcServer,
    user_id: ID,
    request: Request<AcceptRoomInvitationRequest>,
) -> Result<AcceptRoomInvitationResponse, AcceptInviteErr> {
    let req = request.into_inner();
    let room_id = RoomId(req.room_id);

    let mut redis_conn = server.db.redis();

    // Check if room exists
    let room_key_str = room_key(room_id);
    let exists: bool = redis_conn.exists(&room_key_str).await?;
    if !exists {
        return Err(AcceptInviteErr::RoomNotFound);
    }

    // Check if user is already in the room (check before invitation to return correct error)
    let member_key = room_members_key(room_id);
    let is_member: bool = redis_conn.sismember(&member_key, user_id.0).await?;
    if is_member {
        return Err(AcceptInviteErr::AlreadyInRoom);
    }

    // Check if user is invited
    let invitations_key = room_invitations_key(room_id);
    let is_invited: bool = redis_conn.sismember(&invitations_key, user_id.0).await?;
    if !is_invited {
        return Err(AcceptInviteErr::NotInvited);
    }

    // Add user to room members
    let _: usize = redis_conn.sadd(&member_key, user_id.0).await?;

    // Remove from invitations
    let _: usize = redis_conn.srem(&invitations_key, user_id.0).await?;

    Ok(AcceptRoomInvitationResponse { success: true })
}
