use sea_orm::{ConnectionTrait, EntityTrait};

use base::consts::SessionID;
pub mod accept_join_session_invitation;
pub mod add_role;
pub mod allow_user_join_session;
pub mod ban;
pub mod delete_session;
pub mod e2eeize_and_dee2eeize_session;
pub mod get_role;
pub mod get_session_info;
pub mod invite_user_to_session;
pub mod join_session;
pub mod kick;
pub mod leave_session;
pub mod mute;
pub mod new_session;
pub mod session_room_key;
pub mod set_role;
pub mod set_session_info;

async fn query_session(
    id: SessionID,
    db_conn: &impl ConnectionTrait,
) -> Result<Option<entities::session::Model>, sea_orm::DbErr> {
    let ret = entities::session::Entity::find_by_id(id)
        .one(db_conn)
        .await?;
    Ok(ret)
}
