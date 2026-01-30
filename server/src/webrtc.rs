use base::consts::{ID, impl_from_all_ints};
use deadpool_redis::redis::AsyncCommands;
use derive::RedisHset;
use utils::{impl_newtype_int, impl_redis_value_from_for_newint};

use crate::db::redis::redis_key;

impl_newtype_int!(RoomId, u64,);

impl_redis_value_from_for_newint!(RoomId);

impl_from_all_ints!(RoomId);

#[derive(RedisHset)]
pub struct RoomInfo {
    pub title: Option<String>,
    pub room_id: RoomId,
    pub users_num: u32,
    pub auto_delete: bool,
    pub open_join: bool,
    pub creator: ID,
}

pub fn empty_room_name() -> &'static str {
    "webrtc:zero_room"
}

pub fn room_key(room_id: RoomId) -> String {
    redis_key!("webrtc:room:{}", room_id)
}

pub fn room_members_key(room_id: RoomId) -> String {
    redis_key!("webrtc:room:{}:members", room_id)
}

pub fn room_admins_key(room_id: RoomId) -> String {
    redis_key!("webrtc:room:{}:admins", room_id)
}

pub fn room_creator_key(room_id: RoomId) -> String {
    redis_key!("webrtc:room:{}:creator", room_id)
}

pub fn room_invitations_key(room_id: RoomId) -> String {
    redis_key!("webrtc:room:{}:invitations", room_id)
}

pub fn room_joined_users_key(room_id: RoomId) -> String {
    redis_key!("webrtc:room:{}:joined_users", room_id)
}

pub fn room_pending_key(room_id: RoomId) -> String {
    redis_key!("webrtc:room:{}:pending", room_id)
}

/// Check if a user is an admin of a room
pub async fn is_room_admin(
    redis_conn: &mut deadpool_redis::Connection,
    room_id: RoomId,
    user_id: u64,
) -> Result<bool, deadpool_redis::redis::RedisError> {
    let admins_key = room_admins_key(room_id);
    redis_conn.sismember(&admins_key, user_id).await
}

/// Check if a user is the creator of a room
/// The creator is always an admin and cannot be removed
pub async fn is_room_creator(
    redis_conn: &mut deadpool_redis::Connection,
    room_id: RoomId,
    user_id: ID,
) -> anyhow::Result<bool> {
    let creator_key = room_creator_key(room_id);

    // The creator is stored as a string in Redis
    let creator_str: Option<String> = redis_conn.get(&creator_key).await?;
    let creator_str = match creator_str {
        None => {
            anyhow::bail!("Room {} has no creator", room_id);
        }
        Some(v) => v,
    };
    let creator_id: ID = match creator_str.parse() {
        Ok(id) => id,
        Err(_) => {
            anyhow::bail!(
                "Failed to parse creator ID {} for room {}",
                creator_str,
                room_id
            );
        }
    };

    Ok(creator_id == user_id)
}
