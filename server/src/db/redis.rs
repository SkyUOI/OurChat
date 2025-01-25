use base::consts::{ID, SessionID};

pub fn map_mute_to_redis(session: SessionID, user_id: ID) -> String {
    format!("mute:{}:{}", session, user_id)
}

pub fn map_mute_all_to_redis(session: SessionID) -> String {
    format!("mute:{}:all", session)
}

pub fn map_ban_to_redis(session: SessionID, user_id: ID) -> String {
    format!("ban:{}:{}", session, user_id)
}

pub fn map_ban_all_to_redis(session: SessionID) -> String {
    format!("ban:{}:all", session)
}
