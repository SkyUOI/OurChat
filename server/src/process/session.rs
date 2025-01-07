use entities::{role_permissions, user_role_relation};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use crate::consts::{ID, SessionID};
pub mod accept_session;
pub mod add_role;
pub mod get_session_info;
pub mod new_session;
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

async fn check_if_permission_exist(
    user_id: ID,
    permission_checked: u64,
    db_conn: &impl ConnectionTrait,
) -> Result<bool, sea_orm::DbErr> {
    // get all roles first
    let roles = user_role_relation::Entity::find()
        .filter(user_role_relation::Column::UserId.eq(user_id))
        .all(db_conn)
        .await?;
    for i in &roles {
        let permissions_queried = role_permissions::Entity::find()
            .filter(role_permissions::Column::RoleId.eq(i.role_id))
            .all(db_conn)
            .await?;
        for j in permissions_queried {
            if j.permission_id == permission_checked as i64 {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
