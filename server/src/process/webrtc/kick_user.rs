use crate::{
    process::error_msg::{SERVER_ERROR, not_found},
    server::RpcServer,
    webrtc::{
        RoomId, RoomInfo, is_room_admin, is_room_creator, room_admins_key, room_joined_users_key,
        room_key, room_members_key,
    },
};
use base::consts::ID;
use deadpool_redis::redis::AsyncTypedCommands;
use pb::service::ourchat::webrtc::room::kick_user::v1::{
    KickUserFromRoomRequest, KickUserFromRoomResponse,
};
use tonic::{Request, Response, Status};

pub async fn kick_user_from_room(
    server: &RpcServer,
    requester_id: ID,
    request: Request<KickUserFromRoomRequest>,
) -> Result<Response<KickUserFromRoomResponse>, Status> {
    match kick_user_from_room_impl(server, requester_id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            KickErr::Redis(_) | KickErr::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            KickErr::RoomNotFound => Err(Status::not_found(not_found::WEBRTC_ROOM)),
            KickErr::NotAdmin => Err(Status::permission_denied("only room admins can kick users")),
            KickErr::CannotKickCreator => {
                Err(Status::permission_denied("cannot kick room creator"))
            }
            KickErr::CannotKickSelf => Err(Status::invalid_argument("cannot kick yourself")),
            KickErr::UserNotInRoom => Err(Status::not_found("user is not in the room")),
            KickErr::Status(s) => Err(s),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum KickErr {
    #[error("redis error:{0:?}")]
    Redis(#[from] deadpool_redis::redis::RedisError),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
    #[error("room not found")]
    RoomNotFound,
    #[error("not an admin")]
    NotAdmin,
    #[error("cannot kick creator")]
    CannotKickCreator,
    #[error("cannot kick self")]
    CannotKickSelf,
    #[error("user not in room")]
    UserNotInRoom,
    #[error("status error:{0:?}")]
    Status(#[from] Status),
}

async fn kick_user_from_room_impl(
    server: &RpcServer,
    requester_id: ID,
    request: Request<KickUserFromRoomRequest>,
) -> Result<KickUserFromRoomResponse, KickErr> {
    let req = request.into_inner();
    let room_id = RoomId(req.room_id);
    let target_user_id = ID(req.user_id);

    let mut redis_conn = server.db.get_redis_connection().await?;

    // Check if room exists
    let room_key_str = room_key(room_id);
    let exists: bool = redis_conn.exists(&room_key_str).await?;
    if !exists {
        return Err(KickErr::RoomNotFound);
    }

    // Check if requester is an admin
    if !is_room_admin(&mut redis_conn, room_id, *requester_id).await? {
        return Err(KickErr::NotAdmin);
    }

    // Cannot kick yourself
    if requester_id == target_user_id {
        return Err(KickErr::CannotKickSelf);
    }

    // Cannot kick the creator (unless requester is the creator, which is handled by the check above)
    if is_room_creator(&mut redis_conn, room_id, target_user_id).await? {
        return Err(KickErr::CannotKickCreator);
    }

    // Check if target user is in the room
    let member_key = room_members_key(room_id);
    let is_member: bool = redis_conn.sismember(&member_key, *target_user_id).await?;
    if !is_member {
        return Err(KickErr::UserNotInRoom);
    }

    // Remove user from members
    let _: usize = redis_conn.srem(&member_key, *target_user_id).await?;

    // Also remove from admins if they were an admin
    let admins_key = room_admins_key(room_id);
    let _: usize = redis_conn.srem(&admins_key, *target_user_id).await?;

    // Remove from joined_users set and decrement count if they had joined
    let joined_users_key = room_joined_users_key(room_id);
    let was_in_joined: usize = redis_conn.srem(&joined_users_key, *target_user_id).await?;

    if was_in_joined > 0 {
        // User had joined, so decrement the count
        let room_info = RoomInfo::from_redis(&mut redis_conn, &room_key_str).await?;

        // Update user count
        let new_count = room_info.users_num.saturating_sub(1);
        let updated_info = RoomInfo {
            users_num: new_count,
            ..room_info
        };
        let pipe = updated_info.hset_pipe(&room_key_str);
        let _: () = pipe.query_async(&mut redis_conn).await?;
    }

    Ok(KickUserFromRoomResponse { success: true })
}
