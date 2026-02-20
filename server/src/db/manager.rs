use base::constants::{ID, OCID};
use entities::{
    manager_role_relation, prelude::ServerManagementRolePermissions, server_management_role,
    server_management_role_permissions,
};
use migration::predefined::PredefinedServerManagementRole;
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

/// Bootstrap: assigns admin role to a user if configured in settings.
/// This is called on server startup to handle the initial admin account.
///
/// # Arguments
///
/// * `ocid` - The OCID of the user to make admin (optional)
/// * `db_conn` - A reference to the database connection
///
/// # Returns
///
/// * `Result<Option<String>, DbErr>` - The OCID of the user that was made admin, or None if not configured
pub async fn bootstrap_initial_admin(
    ocid: &Option<OCID>,
    db_conn: &impl ConnectionTrait,
) -> Result<Option<OCID>, DbErr> {
    let Some(ocid) = ocid else {
        return Ok(None);
    };

    use entities::prelude::User;
    use entities::user::Column as UserColumn;

    // Find user by OCID
    let user = User::find()
        .filter(UserColumn::Ocid.eq(ocid))
        .one(db_conn)
        .await?;

    let Some(user) = user else {
        tracing::warn!("initial_admin_ocid configured but user not found: {}", ocid);
        return Ok(None);
    };

    let id: ID = user.id.into();
    let already_admin = match manager_role_relation::Entity::find_by_id(id)
        .one(db_conn)
        .await?
    {
        None => false,
        Some(role) => role.role_id == PredefinedServerManagementRole::Admin as i64,
    };

    if already_admin {
        tracing::info!("User {} is already an admin", ocid);
        return Ok(Some(ocid.clone()));
    }

    set_role(id, PredefinedServerManagementRole::Admin as i64, db_conn).await?;
    tracing::info!("Bootstrap: assigned admin role to user with OCID: {}", ocid);
    Ok(Some(ocid.clone()))
}
