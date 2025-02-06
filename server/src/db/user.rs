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
) -> Result<Option<user::Model>, sea_orm::DbErr> {
    User::find_by_id(id).one(db_conn).await
}

/// Get all the friends of a user.
///
/// # Errors
///
/// Fails if any error occurs in the database.
pub async fn get_friends(
    id: ID,
    db_conn: &impl ConnectionTrait,
) -> Result<Vec<friend::Model>, sea_orm::DbErr> {
    let friends = Friend::find()
        .filter(friend::Column::UserId.eq(id))
        .all(db_conn)
        .await?;
    Ok(friends)
}

pub async fn query_contact_user_info(
    id: ID,
    contact_user: ID,
    db_conn: &impl ConnectionTrait,
) -> Result<Option<entities::user_contact_info::Model>, sea_orm::DbErr> {
    let model = entities::user_contact_info::Entity::find_by_id((id.into(), contact_user.into()))
        .one(db_conn)
        .await?;
    Ok(model)
}
