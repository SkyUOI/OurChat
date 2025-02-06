use base::consts::ID;

pub mod accept_friend;
pub mod add_friend;
pub mod delete_friend;
pub mod set_friend_info;

pub fn mapped_add_friend_to_redis(user_id: ID, friend_id: ID) -> String {
    format!("add_friend:{}:{}", user_id, friend_id)
}
