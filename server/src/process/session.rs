use sea_orm::{ConnectionTrait, EntityTrait};

use base::consts::SessionID;
pub mod accept_join_in_session;
pub mod accept_session;
pub mod add_role;
pub mod ban;
pub mod delete_session;
pub mod get_session_info;
pub mod invite_to_session;
pub mod join_in_session;
pub mod leave_session;
pub mod mute;
pub mod new_session;
pub mod session_room_key;
pub mod set_role;
pub mod set_self_info;

async fn query_session(
    id: SessionID,
    db_conn: &impl ConnectionTrait,
) -> Result<Option<entities::session::Model>, sea_orm::DbErr> {
    let ret = entities::session::Entity::find_by_id(id)
        .one(db_conn)
        .await?;
    Ok(ret)
}
