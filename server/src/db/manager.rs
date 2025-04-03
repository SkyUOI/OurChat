use base::consts::ID;
use entities::{manager_role_relation, server_management_role, server_management_role_permissions};
use sea_orm::{QuerySelect, prelude::*};

/// Checks if the manager has the given permission.
///
/// # Arguments
///
/// * `user_id` - The ID of the manager whose permission is to be checked.
/// * `permission_checked` - The permission to be checked.
/// * `db_conn` - A reference to the database connection implementing the `ConnectionTrait`.
///
/// # Returns
///
/// * `Result<bool, sea_orm::DbErr>` - `true` if the user has the given permission, `false` if not, or a `DbErr` if the operation fails.
pub async fn manage_permission_existed(
    user_id: ID,
    permission_checked: i64,
    db_conn: &impl ConnectionTrait,
) -> Result<bool, sea_orm::DbErr> {
    let num = manager_role_relation::Entity::find()
        .join(
            sea_orm::JoinType::InnerJoin,
            manager_role_relation::Relation::ServerManagementRole.def(),
        )
        .join(
            sea_orm::JoinType::InnerJoin,
            server_management_role::Relation::ManagerRoleRelation.def(),
        )
        .filter(manager_role_relation::Column::UserId.eq(user_id))
        .filter(server_management_role_permissions::Column::PermissionId.eq(permission_checked))
        .count(db_conn)
        .await?;
    Ok(num > 0)
}
