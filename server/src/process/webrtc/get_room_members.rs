use crate::{
    process::error_msg::{SERVER_ERROR, not_found},
    server::RpcServer,
    webrtc::{RoomId, room_key, room_members_key},
};
use base::constants::ID;
use pb::service::ourchat::webrtc::room::get_room_members::v1::{
    GetRoomMembersRequest, GetRoomMembersResponse,
};
use redis::AsyncCommands;
use tonic::{Request, Response, Status};

pub async fn get_room_members(
    server: &RpcServer,
    id: ID,
    request: Request<GetRoomMembersRequest>,
) -> Result<Response<GetRoomMembersResponse>, Status> {
    match get_room_members_impl(server, id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            GetRoomMembersErr::Redis(_) | GetRoomMembersErr::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            GetRoomMembersErr::RoomNotFound => Err(Status::not_found(not_found::WEBRTC_ROOM)),
            GetRoomMembersErr::NotInRoom => {
                Err(Status::permission_denied("you are not in this room"))
            }
            GetRoomMembersErr::Status(s) => Err(s),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum GetRoomMembersErr {
    #[error("redis error:{0:?}")]
    Redis(#[from] redis::RedisError),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
    #[error("room not found")]
    RoomNotFound,
    #[error("user not in room")]
    NotInRoom,
    #[error("status error:{0:?}")]
    Status(#[from] Status),
}

async fn get_room_members_impl(
    server: &RpcServer,
    id: ID,
    request: Request<GetRoomMembersRequest>,
) -> Result<GetRoomMembersResponse, GetRoomMembersErr> {
    let req = request.into_inner();
    let room_id = RoomId(req.room_id);

    // Get Redis connection
    let mut redis_conn = server.db.redis();

    // Check if room exists
    let room_key_str = room_key(room_id);
    let exists: bool = redis_conn.exists(&room_key_str).await?;
    if !exists {
        return Err(GetRoomMembersErr::RoomNotFound);
    }

    // Check if the requesting user is a member of the room
    let member_key = room_members_key(room_id);
    let is_member: bool = redis_conn.sismember(&member_key, *id).await?;
    if !is_member {
        return Err(GetRoomMembersErr::NotInRoom);
    }

    // Get all members from the room
    let members: Vec<String> = redis_conn.smembers(&member_key).await?;

    // Parse member IDs
    let member_ids: Vec<u64> = members
        .into_iter()
        .filter_map(|s| s.parse::<u64>().ok())
        .collect();

    let member_count = member_ids.len() as u32;

    Ok(GetRoomMembersResponse {
        success: true,
        member_ids,
        member_count,
    })
}
