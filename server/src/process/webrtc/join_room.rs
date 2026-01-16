use crate::{
    process::error_msg::not_found,
    rabbitmq::WEBRTC_FANOUT_EXCHANGE,
    server::RpcServer,
    webrtc::{
        RoomId, RoomInfo, room_invitations_key, room_joined_users_key, room_key, room_members_key,
    },
};
use base::consts::ID;
use deadpool_lapin::lapin::BasicProperties;
use deadpool_lapin::lapin::options::BasicPublishOptions;
use deadpool_redis::redis::AsyncTypedCommands;
use pb::service::ourchat::webrtc::room::join_room::v1::{JoinRoomRequest, JoinRoomResponse};
use serde::{Deserialize, Serialize};
use tonic::{Request, Response, Status};

#[derive(Serialize, Deserialize)]
struct JoinNotification {
    room_id: u64,
    user_id: u64,
}

pub async fn join_room(
    server: &RpcServer,
    id: ID,
    request: Request<JoinRoomRequest>,
) -> Result<Response<JoinRoomResponse>, Status> {
    match join_room_impl(server, id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            JoinRoomErr::Db(_) | JoinRoomErr::Internal(_) | JoinRoomErr::Redis(_) => {
                tracing::error!("{}", e);
                Err(Status::internal("server error"))
            }
            JoinRoomErr::RoomNotFound => Err(Status::not_found(not_found::WEBRTC_ROOM)),
            JoinRoomErr::NotInvited => Err(Status::permission_denied(
                "you must be invited to join this room",
            )),
            JoinRoomErr::Status(status) => Err(status),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum JoinRoomErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("redis error:{0:?}")]
    Redis(#[from] deadpool_redis::redis::RedisError),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
    #[error("room not found")]
    RoomNotFound,
    #[error("not invited")]
    NotInvited,
}

async fn join_room_impl(
    server: &RpcServer,
    user_id: ID,
    request: Request<JoinRoomRequest>,
) -> Result<JoinRoomResponse, JoinRoomErr> {
    let req = request.into_inner();
    let room_id = RoomId(req.room_id);

    // Check if room exists in Redis
    let mut redis_conn = server.db.get_redis_connection().await?;

    let room_key_str = room_key(room_id);

    // Check if room exists (check if hash has any fields)
    let exists: bool = redis_conn.exists(&room_key_str).await?;
    if !exists {
        return Err(JoinRoomErr::RoomNotFound);
    }

    // Parse room info to check if it's an open join room
    let room_info: RoomInfo = RoomInfo::from_redis(&mut redis_conn, &room_key_str).await?;

    // Check if user is already a member (idempotent operation)
    let member_key = room_members_key(room_id);
    let is_member: bool = redis_conn.sismember(&member_key, *user_id).await?;
    if !is_member {
        // User is not yet a member
        // If the room is not open_join, check if they are invited
        if !room_info.open_join {
            let invitations_key = room_invitations_key(room_id);
            let is_invited: bool = redis_conn.sismember(&invitations_key, *user_id).await?;
            if !is_invited {
                return Err(JoinRoomErr::NotInvited);
            }
            // Remove from invitations since they're now joining
            let _: usize = redis_conn.srem(&invitations_key, *user_id).await?;
        }
    }

    // Add user to room members (idempotent - SADD returns 0 if already present)
    let _: usize = redis_conn.sadd(&member_key, *user_id).await?;

    // Check if user has already joined (for counting purposes)
    let joined_users_key = room_joined_users_key(room_id);
    let has_joined: bool = redis_conn.sismember(&joined_users_key, *user_id).await?;

    if !has_joined {
        // User is joining for the first time, add to joined_users set and increment count
        let _: usize = redis_conn.sadd(&joined_users_key, *user_id).await?;
        let new_count = room_info.users_num + 1;
        let updated_info = RoomInfo {
            users_num: new_count,
            ..room_info
        };
        let pipe = updated_info.hset_pipe(&room_key_str);
        let _: () = pipe.query_async(&mut redis_conn).await?;
    }

    // Get existing members
    let existing_members: std::collections::HashSet<String> =
        redis_conn.smembers(&member_key).await?;
    let existing_users: Vec<u64> = existing_members
        .into_iter()
        .filter_map(|s| s.parse::<u64>().ok())
        .filter(|&id| id != *user_id)
        .collect();

    // Publish join notification to RabbitMQ
    let notification = JoinNotification {
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

    Ok(JoinRoomResponse {
        success: true,
        existing_users,
    })
}
