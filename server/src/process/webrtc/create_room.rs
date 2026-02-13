use crate::{
    helper::generate_webrtc_room_id,
    process::error_msg::SERVER_ERROR,
    server::RpcServer,
    webrtc::{RoomInfo, empty_room_name, room_admins_key, room_creator_key, room_key},
};
use base::constants::ID;
use deadpool_redis::redis::AsyncTypedCommands;
use pb::service::ourchat::webrtc::room::create_room::v1::{CreateRoomRequest, CreateRoomResponse};
use tonic::{Request, Response, Status};

pub async fn create_room(
    server: &RpcServer,
    id: ID,
    request: Request<CreateRoomRequest>,
) -> Result<Response<CreateRoomResponse>, Status> {
    match create_room_impl(server, id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            CreateRoomErr::Db(_) | CreateRoomErr::Internal(_) | CreateRoomErr::Redis(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            CreateRoomErr::Status(status) => Err(status),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum CreateRoomErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("redis error:{0:?}")]
    Redis(#[from] deadpool_redis::redis::RedisError),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
}

async fn create_room_impl(
    server: &RpcServer,
    id: ID,
    request: Request<CreateRoomRequest>,
) -> Result<CreateRoomResponse, CreateRoomErr> {
    let req = request.into_inner();
    let room_id = generate_webrtc_room_id()?;
    let key = room_key(room_id);
    let mut conn = server.db.get_redis_connection().await?;

    let info = RoomInfo {
        title: req.title,
        room_id,
        users_num: 0,
        auto_delete: req.auto_delete,
        open_join: req.open_join,
        creator: id,
    };

    // Set room info
    let pipe = info.hset_pipe(&key);
    let _: () = pipe.query_async(&mut conn).await?;

    // Set creator
    let creator_key = room_creator_key(room_id);
    conn.set(&creator_key, *id).await?;

    // Add creator to admins
    let admins_key = room_admins_key(room_id);
    let _: usize = conn.sadd(&admins_key, *id).await?;

    // Add creator to members
    let members_key = crate::webrtc::room_members_key(room_id);
    let _: usize = conn.sadd(&members_key, *id).await?;

    // Append to empty rooms list
    let _: usize = conn.sadd(empty_room_name(), room_id).await?;

    let ret = CreateRoomResponse { room_id: *room_id };
    Ok(ret)
}
