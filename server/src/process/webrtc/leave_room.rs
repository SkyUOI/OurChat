use crate::{
    rabbitmq::WEBRTC_FANOUT_EXCHANGE,
    server::RpcServer,
    webrtc::{RoomId, RoomInfo, room_joined_users_key, room_key, room_members_key},
};
use base::consts::ID;
use deadpool_lapin::lapin::BasicProperties;
use deadpool_lapin::lapin::options::BasicPublishOptions;
use deadpool_redis::redis::AsyncTypedCommands;
use pb::service::ourchat::webrtc::room::leave_room::v1::{LeaveRoomRequest, LeaveRoomResponse};
use serde::{Deserialize, Serialize};
use tonic::{Request, Response, Status};

#[derive(Serialize, Deserialize)]
struct LeaveNotification {
    room_id: u64,
    user_id: u64,
}

pub async fn leave_room(
    server: &RpcServer,
    id: ID,
    request: Request<LeaveRoomRequest>,
) -> Result<Response<LeaveRoomResponse>, Status> {
    match leave_room_impl(server, id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            LeaveRoomErr::Db(_) | LeaveRoomErr::Internal(_) | LeaveRoomErr::Redis(_) => {
                tracing::error!("{}", e);
                Err(Status::internal("server error"))
            }
            LeaveRoomErr::Status(status) => Err(status),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum LeaveRoomErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("redis error:{0:?}")]
    Redis(#[from] deadpool_redis::redis::RedisError),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
}

async fn leave_room_impl(
    server: &RpcServer,
    user_id: ID,
    request: Request<LeaveRoomRequest>,
) -> Result<LeaveRoomResponse, LeaveRoomErr> {
    let req = request.into_inner();
    let room_id = RoomId(req.room_id);

    let mut redis_conn = server.db.get_redis_connection().await?;

    let room_key_str = room_key(room_id);

    // Remove user from room members (convert ID to u64 for Redis)
    let member_key = room_members_key(room_id);
    let _: usize = redis_conn.srem(&member_key, *user_id).await?;

    // Check if user was in joined_users set (for counting purposes)
    let joined_users_key = room_joined_users_key(room_id);
    let was_in_joined: usize = redis_conn.srem(&joined_users_key, *user_id).await?;

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

    // Publish leave notification to RabbitMQ
    let notification = LeaveNotification {
        room_id: *room_id,
        user_id: *user_id,
    };
    let notification_bytes = serde_json::to_vec(&notification)
        .map_err(|e| anyhow::anyhow!("json serialization error: {}", e))?;

    let rmq_conn = server
        .rabbitmq
        .get()
        .await
        .map_err(|e| anyhow::anyhow!("rabbitmq pool error: {:?}", e))?;
    let channel = rmq_conn
        .create_channel()
        .await
        .map_err(|e| anyhow::anyhow!("rabbitmq channel error: {:?}", e))?;
    let exchange = WEBRTC_FANOUT_EXCHANGE;

    let _ = channel
        .basic_publish(
            exchange,
            "",
            BasicPublishOptions::default(),
            &notification_bytes,
            BasicProperties::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("rabbitmq publish error: {:?}", e))?;

    Ok(LeaveRoomResponse { success: true })
}
