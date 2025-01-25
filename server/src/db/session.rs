use crate::db::redis::{
    map_ban_all_to_redis, map_ban_to_redis, map_mute_all_to_redis, map_mute_to_redis,
};
use anyhow::Context;
use base::consts::{ID, SessionID};
use deadpool_redis::redis::AsyncCommands;
use entities::{role_permissions, session, session_relation, user_role_relation};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DatabaseTransaction, EntityTrait,
    QueryFilter,
};
use std::time::Duration;

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

#[derive(Debug, Clone)]
pub enum MuteStatus {
    RestTime(Duration),
    Permanent,
}

pub async fn user_muted_status(
    user_id: ID,
    session_id: SessionID,
    redis_connection: &mut deadpool_redis::Connection,
) -> Result<Option<MuteStatus>, deadpool_redis::redis::RedisError> {
    let key = map_mute_to_redis(session_id, user_id);
    let user_muted: i64 = redis_connection.ttl(&key).await?;
    let key = map_mute_all_to_redis(session_id);
    let all_muted: i64 = redis_connection.ttl(key).await?;
    let res = user_muted.max(all_muted);
    if res == -2 {
        // not exists
        return Ok(None);
    }
    if res == -1 {
        return Ok(Some(MuteStatus::Permanent));
    }
    Ok(Some(MuteStatus::RestTime(Duration::from_secs(res as u64))))
}

#[derive(Debug, Clone)]
pub enum BanStatus {
    RestTime(Duration),
    Permanent,
}

pub async fn user_banned_status(
    user_id: ID,
    session_id: SessionID,
    redis_connection: &mut deadpool_redis::Connection,
) -> Result<Option<BanStatus>, deadpool_redis::redis::RedisError> {
    let key = map_ban_to_redis(session_id, user_id);
    let user_banned: i64 = redis_connection.ttl(&key).await?;
    let key = map_ban_all_to_redis(session_id);
    let all_banned: i64 = redis_connection.ttl(key).await?;
    let res = user_banned.max(all_banned);
    if res == -2 {
        // not exists
        return Ok(None);
    }
    if res == -1 {
        return Ok(Some(BanStatus::Permanent));
    }
    Ok(Some(BanStatus::RestTime(Duration::from_secs(res as u64))))
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
pub async fn if_permission_exist(
    user_id: ID,
    session_id: SessionID,
    permission_checked: u64,
    db_conn: &impl ConnectionTrait,
) -> Result<bool, sea_orm::DbErr> {
    // get all roles first
    let roles = user_role_relation::Entity::find()
        .filter(user_role_relation::Column::UserId.eq(user_id))
        .filter(user_role_relation::Column::SessionId.eq(session_id))
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

/// Adds a user to a session with a specified role.
///
/// This function inserts a new session relation record and a new user role relation
/// record into the database, associating the given user ID with the specified session ID
/// and role ID.
///
/// # Arguments
///
/// * `session_id` - The ID of the session to which the user is being added.
/// * `id` - The ID of the user being added to the session.
/// * `role` - The role ID to be assigned to the user within the session.
/// If not specified, the default role will be set.
/// * `db_conn` - A reference to the database connection implementing the `ConnectionTrait`.
///
/// # Returns
///
/// * `anyhow::Result<()>` - An empty result if the operation is successful, or an error
///   if the operation fails.
pub async fn join_in_session(
    session_id: SessionID,
    id: ID,
    role: Option<u64>,
    db_conn: &DatabaseTransaction,
) -> anyhow::Result<()> {
    // update the session info
    let mut session_info = get_session_by_id(session_id, db_conn)
        .await?
        .context("not session found")?;
    session_info.size += 1;
    let default_role = session_info.default_role;
    let session_info: session::ActiveModel = session_info.into();
    session_info.update(db_conn).await?;

    let session_relation = session_relation::ActiveModel {
        user_id: ActiveValue::Set(id.into()),
        session_id: ActiveValue::Set(session_id.into()),
        ..Default::default()
    };
    session_relation.insert(db_conn).await?;
    // Add role
    let role_relation = user_role_relation::ActiveModel {
        user_id: ActiveValue::Set(id.into()),
        session_id: ActiveValue::Set(session_id.into()),
        role_id: ActiveValue::Set(role.unwrap_or(default_role as u64) as i64),
    };
    role_relation.insert(db_conn).await?;
    Ok(())
}

/// Adds multiple users to a session with a specified role.
///
/// This function calls the `add_to_session` function for each user ID in the given slice,
/// adding each user to the session with the specified role.
///
/// # Arguments
///
/// * `session_id` - The ID of the session to which the users are being added.
/// * `ids` - A slice of user IDs to add to the session.
/// * `role` - The role ID to be assigned to the users within the session.
/// * `db_conn` - A reference to the database connection implementing the `ConnectionTrait`.
///
/// # Returns
///
/// * `anyhow::Result<()>` - An empty result if the operation is successful, or an error
///   if the operation fails.
pub async fn batch_join_in_session(
    session_id: SessionID,
    ids: &[ID],
    role: Option<u64>,
    db_conn: &DatabaseTransaction,
) -> anyhow::Result<()> {
    for id in ids {
        join_in_session(session_id, *id, role, db_conn).await?;
    }
    Ok(())
}

/// Removes a user from a session and updates the session size.
///
/// This function deletes the user's relation to the session and decrements
/// the session's size in the database. It ensures that both the session relation
/// and the session size are updated atomically within a transaction.
///
/// # Arguments
///
/// * `session_id` - The ID of the session the user is leaving.
/// * `user_id` - The ID of the user who is leaving the session.
/// * `db_conn` - A reference to the database transaction.
///
/// # Returns
///
/// * `Result<(), sea_orm::DbErr>` - An empty result if the operation is successful,
///   or a `DbErr` if the operation fails.
pub async fn leave_session(
    session_id: SessionID,
    user_id: ID,
    db_conn: &DatabaseTransaction,
) -> anyhow::Result<()> {
    session_relation::Entity::delete_by_id((session_id.into(), user_id.into()))
        .exec(db_conn)
        .await?;
    let mut session_info = get_session_by_id(session_id, db_conn)
        .await?
        .context("not session found")?;
    session_info.size -= 1;
    let session_info: session::ActiveModel = session_info.into();
    session_info.update(db_conn).await?;
    Ok(())
}

pub async fn get_session_by_id(
    session_id: SessionID,
    db_conn: &impl ConnectionTrait,
) -> Result<Option<session::Model>, sea_orm::DbErr> {
    session::Entity::find_by_id(session_id).one(db_conn).await
}
