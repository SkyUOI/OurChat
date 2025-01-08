use anyhow::Context;
use entities::{friend, prelude::*, user};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use base::consts::ID;

/// Get the user info from the database by id.
///
/// # Errors
///
/// Fails if any error occurs in the database.
pub async fn get_account_info_db(
    id: ID,
    db_conn: &impl ConnectionTrait,
) -> anyhow::Result<Option<user::Model>> {
    Ok(User::find_by_id(id).one(db_conn).await?)
}

/// Get all the friends of a user.
///
/// # Errors
///
/// Fails if any error occurs in the database.
pub async fn get_friends(
    id: ID,
    db_conn: &impl ConnectionTrait,
) -> anyhow::Result<Vec<friend::Model>> {
    let friends = Friend::find()
        .filter(friend::Column::UserId.eq(id))
        .all(db_conn)
        .await?;
    Ok(friends)
}

/// Get a specific friend of a user by user id and friend id.
///
/// # Errors
///
/// Fails if any error occurs in the database, or if the friend is not found.
pub async fn get_one_friend(
    id: ID,
    friend_id: ID,
    db_conn: &impl ConnectionTrait,
) -> anyhow::Result<Option<friend::Model>> {
    let friend = Friend::find()
        .filter(friend::Column::UserId.eq(id))
        .filter(friend::Column::FriendId.eq(friend_id))
        .one(db_conn)
        .await
        .with_context(|| {
            format!(
                "Failed to get the friend of user {} and friend {}",
                id, friend_id
            )
        })?;
    Ok(friend)
}
