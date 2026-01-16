use std::str::FromStr;

use anyhow::Context;
use base::consts::impl_from_all_ints;
use deadpool_redis::redis::{AsyncCommands, FromRedisValue, ToRedisArgs};
use derive::RedisHset;
use utils::impl_newtype_int;

impl_newtype_int!(RoomId, u64,);

impl ToRedisArgs for RoomId {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + deadpool_redis::redis::RedisWrite,
    {
        out.write_arg(self.0.to_string().as_bytes());
    }
}

impl FromRedisValue for RoomId {
    fn from_redis_value(
        v: &deadpool_redis::redis::Value,
    ) -> deadpool_redis::redis::RedisResult<Self> {
        let s: String = FromRedisValue::from_redis_value(v)?;
        let num: u64 = s.parse().map_err(|_| {
            deadpool_redis::redis::RedisError::from((
                deadpool_redis::redis::ErrorKind::TypeError,
                "Failed to parse RoomId from string",
            ))
        })?;
        Ok(RoomId(num))
    }
}

impl FromStr for RoomId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(RoomId(s.parse::<u64>().context("not valid RoomId")?))
    }
}

impl_from_all_ints!(RoomId);

#[derive(RedisHset)]
pub struct RoomInfo {
    pub title: Option<String>,
    pub room_id: RoomId,
    pub users_num: u32,
    pub auto_delete: bool,
    pub open_join: bool,
    pub creator: u64,
}

pub fn empty_room_name() -> &'static str {
    "webrtc:zero_room"
}

pub fn room_key(room_id: RoomId) -> String {
    format!("webrtc:room:{}", room_id)
}

pub fn room_members_key(room_id: RoomId) -> String {
    format!("webrtc:room:{}:members", room_id)
}

pub fn room_admins_key(room_id: RoomId) -> String {
    format!("webrtc:room:{}:admins", room_id)
}

pub fn room_creator_key(room_id: RoomId) -> String {
    format!("webrtc:room:{}:creator", room_id)
}

pub fn room_invitations_key(room_id: RoomId) -> String {
    format!("webrtc:room:{}:invitations", room_id)
}

pub fn room_joined_users_key(room_id: RoomId) -> String {
    format!("webrtc:room:{}:joined_users", room_id)
}

pub fn room_pending_key(room_id: RoomId) -> String {
    format!("webrtc:room:{}:pending", room_id)
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
    user_id: u64,
) -> Result<bool, deadpool_redis::redis::RedisError> {
    let creator_key = room_creator_key(room_id);

    // The creator is stored as a string in Redis
    let creator_str: String = redis_conn
        .get(&creator_key)
        .await
        .unwrap_or(None)
        .unwrap_or_default();
    let creator_id: u64 = creator_str.parse().unwrap_or(0);

    Ok(creator_id == user_id)
}
