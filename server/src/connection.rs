//! Process the connection to server

mod basic;
pub mod process;

use crate::{
    consts::ID,
    entities::{operations, prelude::*},
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

pub mod db {
    pub use super::basic::get_id;
    pub use super::process::new_session::{add_to_session, batch_add_to_session, create_session};
}

struct UserInfo {
    id: ID,
    ocid: String,
}

async fn get_requests(id: ID, db_conn: &DatabaseConnection) -> anyhow::Result<Vec<String>> {
    let id: u64 = id.into();
    let stored_requests = Operations::find()
        .filter(operations::Column::UserId.eq(id))
        .all(db_conn)
        .await?;
    let mut ret = Vec::new();
    for i in stored_requests {
        if i.once {
            Operations::delete_by_id(i.oper_id).exec(db_conn).await?;
        }
        if i.expires_at < chrono::Utc::now() {
            Operations::delete_by_id(i.oper_id).exec(db_conn).await?;
            continue;
        }
        ret.push(i.operation);
    }
    Ok(ret)
}
