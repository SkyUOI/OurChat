use base::consts::{ID, SessionID};
use migration::m20241229_022701_add_role_for_session::PredefinedRoles;
use sea_orm::{ActiveModelTrait, ActiveValue, ConnectionTrait, EntityTrait};

use crate::{db, helper};

pub async fn query_friend(
    user_id: ID,
    friend_id: ID,
    db_conn: &impl ConnectionTrait,
) -> Result<Option<entities::friend::Model>, sea_orm::DbErr> {
    let ret = entities::friend::Entity::find_by_id((user_id.into(), friend_id.into()))
        .one(db_conn)
        .await?;
    Ok(ret)
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteFriendError {
    #[error("session not found")]
    FriendShipNotFound,
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
}

/// Deletes the friend relationship between `user_id` and `friend_id`
///
/// This function will try to delete the session between `user_id` and `friend_id`
/// and the relations of `user_id` and `friend_id` in the `friends` table.
///
/// If the session is not found, it will return `Ok(())` directly.
///
/// # Errors
///
/// Returns an error if any database error occurs.
pub async fn delete_friend(
    user_id: ID,
    friend_id: ID,
    db_conn: &impl ConnectionTrait,
) -> Result<(), DeleteFriendError> {
    // Not friend
    if let Some(model) = entities::friend::Entity::find_by_id((user_id.into(), friend_id.into()))
        .one(db_conn)
        .await?
    {
        let session_id = model.session_id;
        // delete session
        if let Err(e) = db::session::delete_session(session_id.into(), db_conn).await {
            match e {
                db::session::SessionError::SessionNotFound => {
                    tracing::error!("Session of friend is not found");
                }
                db::session::SessionError::Db(db_err) => {
                    Err(db_err)?;
                }
            }
        }
        // delete relations
        entities::friend::Entity::delete_by_id((user_id.into(), friend_id.into()))
            .exec(db_conn)
            .await?;
        entities::friend::Entity::delete_by_id((friend_id.into(), user_id.into()))
            .exec(db_conn)
            .await?;
    } else {
        return Err(DeleteFriendError::FriendShipNotFound);
    }
    Ok(())
}

/// Creates a new session between `user_id1` and `user_id2` and adds friend relations.
///
/// # Errors
///
/// Returns an error if:
///
/// - `user_id1` or `user_id2` is unknown
/// - `display_name1` or `display_name2` cannot be set
/// - any other database error occurs
///
/// # Panics
///
/// Panics if `user_id1` equals `user_id2`
pub async fn add_friend(
    user_id1: ID,
    user_id2: ID,
    display_name1: Option<String>,
    display_name2: Option<String>,
    transaction: &sea_orm::DatabaseTransaction,
) -> anyhow::Result<SessionID> {
    // create a session
    let session_id = helper::generate_session_id()?;
    db::session::create_session_db(session_id, 0, "".to_owned(), transaction, false).await?;
    db::session::batch_join_in_session(
        session_id,
        &[user_id1, user_id2],
        Some(PredefinedRoles::Owner.into()),
        transaction,
    )
    .await?;

    // create friend relations
    let add = async |a: ID, b: ID| {
        let model = entities::friend::ActiveModel {
            user_id: ActiveValue::Set(a.into()),
            friend_id: ActiveValue::Set(b.into()),
            session_id: ActiveValue::Set(session_id.into()),
        };
        model.insert(transaction).await
    };
    add(user_id1, user_id2).await?;
    add(user_id2, user_id1).await?;
    let display_name_set = async |a: ID, b: ID, display_name| {
        let model = entities::user_contact_info::ActiveModel {
            user_id: ActiveValue::Set(a.into()),
            contact_user_id: ActiveValue::Set(b.into()),
            display_name: ActiveValue::Set(display_name),
        };
        model.insert(transaction).await
    };
    display_name_set(user_id1, user_id2, display_name2).await?;
    display_name_set(user_id2, user_id1, display_name1).await?;
    Ok(session_id)
}
