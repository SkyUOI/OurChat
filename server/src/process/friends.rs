use base::constants::ID;

use crate::db::redis_mappings::redis_key;

pub mod accept_friend_invitation;
pub mod add_friend;
pub mod delete_friend;
pub mod set_friend_info;

pub fn mapped_add_friend_to_redis(user_id: ID, friend_id: ID) -> String {
    redis_key!("add_friend:{user_id}:{friend_id}")
}
