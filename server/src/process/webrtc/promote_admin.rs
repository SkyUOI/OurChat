use crate::{
    process::error_msg::{SERVER_ERROR, not_found},
    server::RpcServer,
    webrtc::{RoomId, is_room_admin, room_admins_key, room_key, room_members_key},
};
use base::consts::ID;
use deadpool_redis::redis::AsyncTypedCommands;
use pb::service::ourchat::webrtc::room::promote_admin::v1::{
    PromoteRoomAdminRequest, PromoteRoomAdminResponse,
};
use tonic::{Request, Response, Status};

pub async fn promote_room_admin(
    server: &RpcServer,
    requester_id: ID,
    request: Request<PromoteRoomAdminRequest>,
) -> Result<Response<PromoteRoomAdminResponse>, Status> {
    match promote_room_admin_impl(server, requester_id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            PromoteAdminErr::Redis(_) | PromoteAdminErr::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            PromoteAdminErr::RoomNotFound => Err(Status::not_found(not_found::WEBRTC_ROOM)),
            PromoteAdminErr::NotAdmin => Err(Status::permission_denied(
                "only room admins can promote other users",
            )),
            PromoteAdminErr::NotInRoom => Err(Status::not_found("user is not in the room")),
            PromoteAdminErr::Status(s) => Err(s),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum PromoteAdminErr {
    #[error("redis error:{0:?}")]
    Redis(#[from] deadpool_redis::redis::RedisError),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
    #[error("room not found")]
    RoomNotFound,
    #[error("not an admin")]
    NotAdmin,
    #[error("user not in room")]
    NotInRoom,
    #[error("status error:{0:?}")]
    Status(#[from] Status),
}

async fn promote_room_admin_impl(
    server: &RpcServer,
    requester_id: ID,
    request: Request<PromoteRoomAdminRequest>,
) -> Result<PromoteRoomAdminResponse, PromoteAdminErr> {
    let req = request.into_inner();
    let room_id = RoomId(req.room_id);
    let target_user_id = ID(req.user_id);

    let mut redis_conn = server.db.get_redis_connection().await?;

    // Check if room exists
    let room_key_str = room_key(room_id);
    let exists: bool = redis_conn.exists(&room_key_str).await?;
    if !exists {
        return Err(PromoteAdminErr::RoomNotFound);
    }

    // Check if requester is an admin
    if !is_room_admin(&mut redis_conn, room_id, *requester_id).await? {
        return Err(PromoteAdminErr::NotAdmin);
    }

    // Check if target user is in the room
    let member_key = room_members_key(room_id);
    let is_member: bool = redis_conn.sismember(&member_key, *target_user_id).await?;
    if !is_member {
        return Err(PromoteAdminErr::NotInRoom);
    }

    // Promote user to admin
    let admins_key = room_admins_key(room_id);
    let _: usize = redis_conn.sadd(&admins_key, *target_user_id).await?;

    Ok(PromoteRoomAdminResponse { success: true })
}
