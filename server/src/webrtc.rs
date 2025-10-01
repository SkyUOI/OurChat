use std::str::FromStr;

use anyhow::Context;
use base::consts::impl_from_all_ints;
use deadpool_redis::redis::{FromRedisValue, ToRedisArgs};
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
}

pub fn empty_room_name() -> &'static str {
    "webrtc:zero_room"
}

pub fn room_key(room_id: RoomId) -> String {
    format!("webrtc:room:{}", room_id)
}
