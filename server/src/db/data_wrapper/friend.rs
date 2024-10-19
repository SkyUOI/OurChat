use crate::{consts::ID, entities};

#[derive(Debug)]
pub struct Friend {
    pub id: ID,
    pub friend_id: ID,
    pub display_name: String,
}

impl From<entities::sqlite::friend::Model> for Friend {
    fn from(value: entities::sqlite::friend::Model) -> Self {
        Self {
            id: value.user_id.into(),
            friend_id: value.friend_id.into(),
            display_name: value.name,
        }
    }
}

impl From<entities::mysql::friend::Model> for Friend {
    fn from(value: entities::mysql::friend::Model) -> Self {
        Self {
            id: value.user_id.into(),
            friend_id: value.friend_id.into(),
            display_name: value.name,
        }
    }
}
