use crate::{
    helper::GENERATOR,
    process::error_msg::SERVER_ERROR,
    server::RpcServer,
    webrtc::{RoomId, RoomInfo, empty_room_name},
};
use anyhow::Context;
use base::consts::ID;
use deadpool_redis::redis::AsyncTypedCommands;
use pb::service::ourchat::webrtc::room::create_room::v1::{CreateRoomRequest, CreateRoomResponse};
use snowdon::ClassicLayoutSnowflakeExtension;
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

fn room_key(room_id: RoomId) -> String {
    format!("webrtc:room:{}", room_id)
}

async fn create_room_impl(
    server: &RpcServer,
    _id: ID,
    request: Request<CreateRoomRequest>,
) -> Result<CreateRoomResponse, CreateRoomErr> {
    let req = request.into_inner();
    let room_id = RoomId(
        GENERATOR
            .generate()
            .context("failed to generate snowflake id")?
            .into_i64()
            .try_into()
            .context("failed to generate snowflake id")?,
    );
    let key = room_key(room_id);
    let mut conn = server
        .db
        .redis_pool
        .get()
        .await
        .context("cannot get redis connection")?;
    let info = RoomInfo {
        title: req.title,
        room_id,
        users_num: 0,
        auto_delete: req.auto_delete,
    };
    let pipe = info.hset_pipe(&key);
    let _: () = pipe.query_async(&mut conn).await?;
    // append new created
    conn.sadd(empty_room_name(), room_id).await?;
    let ret = CreateRoomResponse { room_id: *room_id };
    Ok(ret)
}
