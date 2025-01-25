use entities::{role_permissions, session_relation, user_role_relation};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use base::consts::{ID, SessionID};

/// Retrieves all session relations associated with the given user ID.
///
/// # Arguments
///
/// * `user_id` - The ID of the user whose session relations are to be fetched.
/// * `db_conn` - A reference to the database connection implementing the `ConnectionTrait`.
///
/// # Returns
///
/// * `Result<Vec<session_relation::Model>, sea_orm::DbErr>` - A vector of `session_relation::Model`
///   objects representing the session relations for the specified user, or a `DbErr` if the
///   operation fails.
pub async fn get_all_session_relations(
    user_id: ID,
    db_conn: &impl ConnectionTrait,
) -> Result<Vec<session_relation::Model>, sea_orm::DbErr> {
    let ret = session_relation::Entity::find()
        .filter(session_relation::Column::UserId.eq(user_id))
        .all(db_conn)
        .await?;
    Ok(ret)
}

/// Retrieves all members of the specified session.
///
/// # Arguments
///
/// * `session_id` - The ID of the session whose members are to be fetched.
/// * `db_conn` - A reference to the database connection implementing the `ConnectionTrait`.
///
/// # Returns
///
/// * `Result<Vec<session_relation::Model>, sea_orm::DbErr>` - A vector of `session_relation::Model`
///   objects representing the members of the specified session, or a `DbErr` if the operation
///   fails.
pub async fn get_members(
    session_id: SessionID,
    db_conn: &impl ConnectionTrait,
) -> Result<Vec<session_relation::Model>, sea_orm::DbErr> {
    let users = session_relation::Entity::find()
        .filter(session_relation::Column::SessionId.eq(session_id))
        .all(db_conn)
        .await?;
    Ok(users)
}

/// Retrieves the users of the specified session and the specified role.
///
/// # Arguments
///
/// * `role` - The role of the users to be fetched.
/// * `session_id` - The ID of the session whose owner is to be fetched.
/// * `db_conn` - A reference to the database connection implementing the `ConnectionTrait`.
///
/// # Returns
///
/// * `Result<Vec<user_role_relation::Model>, sea_orm::DbErr>` - A vector of `user_role_relation::Model`
///   objects representing the users of the specified session, or a `DbErr` if the operation
///   fails.
pub async fn query_session_role(
    session_id: SessionID,
    role: u64,
    db_conn: &impl ConnectionTrait,
) -> Result<Vec<user_role_relation::Model>, sea_orm::DbErr> {
    let ret = user_role_relation::Entity::find()
        .filter(user_role_relation::Column::SessionId.eq(session_id))
        .filter(user_role_relation::Column::RoleId.eq(role))
        .all(db_conn)
        .await?;
    Ok(ret)
}

/// Retrieves all roles of the specified session.
///
/// # Arguments
///
/// * `session_id` - The ID of the session whose roles are to be fetched.
/// * `db_conn` - A reference to the database connection implementing the `ConnectionTrait`.
///
/// # Returns
///
/// * `Result<Vec<user_role_relation::Model>, sea_orm::DbErr>` - A vector of `user_role_relation::Model`
///   objects representing the roles of the specified session, or a `DbErr` if the operation
///   fails.
pub async fn get_all_roles_of_session(
    session_id: SessionID,
    db_conn: &impl ConnectionTrait,
) -> Result<Vec<user_role_relation::Model>, sea_orm::DbErr> {
    let ret = user_role_relation::Entity::find()
        .filter(user_role_relation::Column::SessionId.eq(session_id))
        .all(db_conn)
        .await?;
    Ok(ret)
}

/// Checks if the given user is in the given session.
///
/// # Arguments
///
/// * `user_id` - The ID of the user to check.
/// * `session_id` - The ID of the session to check.
/// * `db_conn` - A reference to the database connection implementing the `ConnectionTrait`.
///
/// # Returns
///
/// * `Result<bool, sea_orm::DbErr>` - `true` if the user is in the session, `false` if not, or a `DbErr` if the operation fails.
pub async fn check_user_in_session(
    user_id: ID,
    session_id: SessionID,
    db_conn: &impl ConnectionTrait,
) -> Result<bool, sea_orm::DbErr> {
    let ret = session_relation::Entity::find_by_id((user_id.into(), session_id.into()))
        .one(db_conn)
        .await?;
    Ok(ret.is_some())
}

/// Checks if the user has the given permission.
///
/// # Arguments
///
/// * `user_id` - The ID of the user whose permission is to be checked.
/// * `permission_checked` - The permission to be checked.
/// * `db_conn` - A reference to the database connection implementing the `ConnectionTrait`.
///
/// # Returns
///
/// * `Result<bool, sea_orm::DbErr>` - `true` if the user has the given permission, `false` if not, or a `DbErr` if the operation fails.
pub async fn check_if_permission_exist(
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
