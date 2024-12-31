use sea_orm::{ConnectionTrait, EntityTrait};

use crate::consts::SessionID;
pub mod accept_session;
pub mod get_session_info;
pub mod new_session;
pub mod set_session_info;

async fn query_session(
    id: SessionID,
    db_conn: &impl ConnectionTrait,
) -> Result<Option<entities::session::Model>, sea_orm::DbErr> {
    let id: i64 = id.into();
    let ret = entities::session::Entity::find_by_id(id)
        .one(db_conn)
        .await?;
    Ok(ret)
}
