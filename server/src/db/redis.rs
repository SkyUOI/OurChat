use base::consts::{ID, SessionID};

use crate::SERVER_INFO;

/// Macro to create Redis keys with server_name as prefix efficiently.
///
/// Uses `fmt::Arguments` to avoid double String allocation.
///
/// # Example
/// ```ignore
/// redis_key!("mute:{}", session_id)           // -> "server123:mute:456"
/// redis_key!("ban:{}:{}", session, user_id)   // -> "server123:ban:456:789"
/// ```
pub macro redis_key($($arg:tt)*) {
    format!("{}:{}", SERVER_INFO.server_name, format_args!($($arg)*))
}

pub fn map_mute_to_redis(session: SessionID, user_id: ID) -> String {
    redis_key!("mute:{session}:{user_id}")
}

pub fn map_mute_all_to_redis(session: SessionID) -> String {
    redis_key!("mute:{session}:all")
}

pub fn map_ban_to_redis(session: SessionID, user_id: ID) -> String {
    redis_key!("ban:{session}:{user_id}")
}

pub fn map_ban_all_to_redis(session: SessionID) -> String {
    redis_key!("ban:{session}:all")
}

pub fn map_server_ban_to_redis(user_id: ID) -> String {
    redis_key!("server_ban:{user_id}")
}

pub fn map_failed_login_to_redis(user_id: ID) -> String {
    redis_key!("failed_login:{user_id}")
}
