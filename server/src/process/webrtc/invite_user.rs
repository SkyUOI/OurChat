use crate::{
    process::error_msg::{SERVER_ERROR, not_found},
    server::RpcServer,
    webrtc::{RoomId, is_room_admin, room_invitations_key, room_key},
};
use base::constants::ID;
use deadpool_redis::redis::AsyncTypedCommands;
use pb::service::ourchat::webrtc::room::invite_user::v1::{
    InviteUserToRoomRequest, InviteUserToRoomResponse,
};
use tonic::{Request, Response, Status};

pub async fn invite_user_to_room(
    server: &RpcServer,
    requester_id: ID,
    request: Request<InviteUserToRoomRequest>,
) -> Result<Response<InviteUserToRoomResponse>, Status> {
    match invite_user_to_room_impl(server, requester_id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            InviteErr::Redis(_) | InviteErr::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            InviteErr::RoomNotFound => Err(Status::not_found(not_found::WEBRTC_ROOM)),
            InviteErr::NotAdmin => Err(Status::permission_denied(
                "only room admins can invite users",
            )),
            InviteErr::AlreadyInvited => Err(Status::already_exists(
                "user is already invited to this room",
            )),
            InviteErr::Status(s) => Err(s),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum InviteErr {
    #[error("redis error:{0:?}")]
    Redis(#[from] deadpool_redis::redis::RedisError),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
    #[error("room not found")]
    RoomNotFound,
    #[error("not an admin")]
    NotAdmin,
    #[error("user already invited")]
    AlreadyInvited,
    #[error("status error:{0:?}")]
    Status(#[from] Status),
}

async fn invite_user_to_room_impl(
    server: &RpcServer,
    requester_id: ID,
    request: Request<InviteUserToRoomRequest>,
) -> Result<InviteUserToRoomResponse, InviteErr> {
    let req = request.into_inner();
    let room_id = RoomId(req.room_id);
    let target_user_id = req.user_id;

    let mut redis_conn = server.db.get_redis_connection().await?;

    // Check if room exists
    let room_key_str = room_key(room_id);
    let exists: bool = redis_conn.exists(&room_key_str).await?;
    if !exists {
        return Err(InviteErr::RoomNotFound);
    }

    // Check if requester is an admin
    if !is_room_admin(&mut redis_conn, room_id, *requester_id).await? {
        return Err(InviteErr::NotAdmin);
    }

    // Check if user is already invited
    let invitations_key = room_invitations_key(room_id);
    let is_invited: bool = redis_conn
        .sismember(&invitations_key, target_user_id)
        .await?;
    if is_invited {
        return Err(InviteErr::AlreadyInvited);
    }

    // Add user to invitations
    let _: usize = redis_conn.sadd(&invitations_key, target_user_id).await?;

    Ok(InviteUserToRoomResponse { success: true })
}
