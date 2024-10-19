use crate::{
    consts::{ID, TimeStamp},
    entities,
};

#[derive(Debug)]
pub struct User {
    pub email: String,
    pub time: TimeStamp,
    pub public_update_time: TimeStamp,
    pub update_time: TimeStamp,
    pub name: String,
    pub ocid: String,
    pub id: ID,
}

impl From<entities::mysql::user::Model> for User {
    fn from(value: entities::mysql::user::Model) -> Self {
        Self {
            email: value.email,
            time: value.time,
            public_update_time: value.public_update_time,
            update_time: value.update_time,
            name: value.name,
            ocid: value.ocid,
            id: ID::from(value.id),
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
            name: value.name,
            ocid: value.ocid,
            id: ID::from(value.id),
        }
    }
}
