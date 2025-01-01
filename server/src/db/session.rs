use entities::{session_relation, user_role_relation};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use crate::consts::{ID, SessionID};

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
    let id: u64 = user_id.into();
    let ret = session_relation::Entity::find()
        .filter(session_relation::Column::UserId.eq(id))
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
    let id: u64 = session_id.into();
    let users = session_relation::Entity::find()
        .filter(session_relation::Column::SessionId.eq(id))
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
    let id: u64 = session_id.into();
    let ret = user_role_relation::Entity::find()
        .filter(user_role_relation::Column::SessionId.eq(id))
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
    let id: u64 = session_id.into();
    let ret = user_role_relation::Entity::find()
        .filter(user_role_relation::Column::SessionId.eq(id))
        .all(db_conn)
        .await?;
    Ok(ret)
}
