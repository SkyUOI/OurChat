use crate::{consts::TimeStamp, entities};

pub struct User {
    pub email: String,
    pub time: TimeStamp,
    pub public_update_time: TimeStamp,
    pub update_time: TimeStamp,
}

impl From<entities::mysql::user::Model> for User {
    fn from(value: entities::mysql::user::Model) -> Self {
        Self {
            email: value.email,
            time: value.time,
            public_update_time: value.public_update_time,
            update_time: value.update_time,
        }
    }
}

impl From<entities::sqlite::user::Model> for User {
    fn from(value: entities::sqlite::user::Model) -> Self {
        Self {
            email: value.email,
            time: value.time,
            public_update_time: value.public_update_time,
            update_time: value.update_time,
        }
    }
}
