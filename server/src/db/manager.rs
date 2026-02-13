use base::constants::ID;
use entities::{
    manager_role_relation, prelude::ServerManagementRolePermissions, server_management_role,
    server_management_role_permissions,
};
use sea_orm::{DatabaseTransaction, QuerySelect, prelude::*};

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
) -> Result<bool, DbErr> {
    let num = manager_role_relation::Entity::find()
        .join(
            sea_orm::JoinType::InnerJoin,
            manager_role_relation::Relation::ServerManagementRole.def(),
        )
        .join(
            sea_orm::JoinType::InnerJoin,
            server_management_role::Relation::ServerManagementRolePermissions.def(),
        )
        .filter(manager_role_relation::Column::UserId.eq(user_id))
        .filter(server_management_role_permissions::Column::PermissionId.eq(permission_checked))
        .one(db_conn)
        .await?;
    Ok(num.is_some())
}

/// Adds a role for server management.
///
/// # Arguments
///
/// * `name` - The name of the role.
/// * `description` - The description of the role.
/// * `permissions` - The permissions of the role.
/// * `db_conn` - A reference to the database transaction implementing the `ConnectionTrait`.
///
/// # Returns
///
/// * `Result<server_management_role::Model, sea_orm::DbErr>` - The created role model if the operation is successful, or a `DbErr` if the operation fails.
pub async fn add_role(
    name: String,
    description: Option<String>,
    permissions: impl IntoIterator<Item = i64>,
    db_conn: &DatabaseTransaction,
) -> Result<server_management_role::Model, DbErr> {
    let role = server_management_role::ActiveModel {
        name: sea_orm::ActiveValue::Set(name),
        description: sea_orm::ActiveValue::Set(description),
        ..Default::default()
    };
    let model = role.insert(db_conn).await?;
    ServerManagementRolePermissions::insert_many(permissions.into_iter().map(|x| {
        server_management_role_permissions::ActiveModel {
            permission_id: sea_orm::ActiveValue::Set(x),
            role_id: sea_orm::ActiveValue::Set(model.id),
        }
    }))
    .exec(db_conn)
    .await?;
    Ok(model)
}

/// Sets the role for a user.
///
/// # Arguments
///
/// * `user_id` - The ID of the user to set the role for.
/// * `role_id` - The ID of the role to set for the user.
/// * `db_conn` - A reference to the database connection implementing the `ConnectionTrait`.
///
/// # Returns
///
/// * `Result<(), sea_orm::DbErr>` - An empty result if the operation is successful, or a `DbErr` if the operation fails.
pub async fn set_role(
    user_id: ID,
    role_id: i64,
    db_conn: &impl ConnectionTrait,
) -> Result<(), DbErr> {
    manager_role_relation::ActiveModel {
        user_id: sea_orm::ActiveValue::Set(user_id.into()),
        role_id: sea_orm::ActiveValue::Set(role_id),
    }
    .insert(db_conn)
    .await?;
    Ok(())
}

/// Removes a server management role from a user.
///
/// # Arguments
///
/// * `user_id` - The ID of the user to remove the role from.
/// * `role_id` - The ID of the role to remove from the user.
/// * `db_conn` - A reference to the database connection implementing the `ConnectionTrait`.
///
/// # Returns
///
/// * `Result<(), sea_orm::DbErr>` - An empty result if the operation is successful, or a `DbErr` if the operation fails.
pub async fn remove_role(
    user_id: ID,
    role_id: i64,
    db_conn: &impl ConnectionTrait,
) -> Result<(), DbErr> {
    manager_role_relation::Entity::delete_many()
        .filter(manager_role_relation::Column::UserId.eq(user_id))
        .filter(manager_role_relation::Column::RoleId.eq(role_id))
        .exec(db_conn)
        .await?;
    Ok(())
}
