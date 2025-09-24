mod move_room;

use deadpool_redis::redis::ToRedisArgs;
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

pub struct RoomInfo {
    pub title: Option<String>,
    pub room_id: RoomId,
    pub users_num: u32,
    pub auto_delete: bool,
}

pub fn zero_room_name() -> &'static str {
    "webrtc:zero_room"
}
