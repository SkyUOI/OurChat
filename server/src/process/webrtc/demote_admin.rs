use crate::{
    process::error_msg::{SERVER_ERROR, not_found},
    server::RpcServer,
    webrtc::{RoomId, is_room_creator, room_admins_key, room_key},
};
use base::constants::ID;
use pb::service::ourchat::webrtc::room::demote_admin::v1::{
    DemoteRoomAdminRequest, DemoteRoomAdminResponse,
};
use redis::AsyncTypedCommands;
use tonic::{Request, Response, Status};

pub async fn demote_room_admin(
    server: &RpcServer,
    requester_id: ID,
    request: Request<DemoteRoomAdminRequest>,
) -> Result<Response<DemoteRoomAdminResponse>, Status> {
    match demote_room_admin_impl(server, requester_id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            DemoteAdminErr::Redis(_) | DemoteAdminErr::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            DemoteAdminErr::RoomNotFound => Err(Status::not_found(not_found::WEBRTC_ROOM)),
            DemoteAdminErr::NotCreator => Err(Status::permission_denied(
                "only room creator can demote admins",
            )),
            DemoteAdminErr::CannotDemoteCreator => {
                Err(Status::permission_denied("cannot demote room creator"))
            }
            DemoteAdminErr::Status(s) => Err(s),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum DemoteAdminErr {
    #[error("redis error:{0:?}")]
    Redis(#[from] redis::RedisError),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
    #[error("room not found")]
    RoomNotFound,
    #[error("not creator")]
    NotCreator,
    #[error("cannot demote creator")]
    CannotDemoteCreator,
    #[error("status error:{0:?}")]
    Status(#[from] Status),
}

async fn demote_room_admin_impl(
    server: &RpcServer,
    requester_id: ID,
    request: Request<DemoteRoomAdminRequest>,
) -> Result<DemoteRoomAdminResponse, DemoteAdminErr> {
    let req = request.into_inner();
    let room_id = RoomId(req.room_id);
    let target_user_id = ID(req.user_id);

    let mut redis_conn = server.db.redis();

    // Check if room exists
    let room_key_str = room_key(room_id);
    let exists: bool = redis_conn.exists(&room_key_str).await?;
    if !exists {
        return Err(DemoteAdminErr::RoomNotFound);
    }

    // Only creator can demote admins
    if !is_room_creator(&mut redis_conn, room_id, requester_id).await? {
        return Err(DemoteAdminErr::NotCreator);
    }

    // Cannot demote the creator
    if is_room_creator(&mut redis_conn, room_id, target_user_id).await? {
        return Err(DemoteAdminErr::CannotDemoteCreator);
    }

    // Demote user from admin
    let admins_key = room_admins_key(room_id);
    let _: usize = redis_conn.srem(&admins_key, *target_user_id).await?;

    Ok(DemoteRoomAdminResponse { success: true })
}
