use base::time::TimeStamp;
use entities::{prelude::UserChatMsg, user_chat_msg};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ConnectionTrait, DatabaseBackend, EntityTrait, ModelTrait,
    Paginator, PaginatorTrait, Statement,
};

use base::consts::{ID, MsgID};

#[derive(Debug, thiserror::Error)]
pub enum MsgError {
    #[error("database error:{0:?}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("unknown error:{0:?}")]
    UnknownError(#[from] anyhow::Error),
    #[error("Don't have privilege")]
    WithoutPrivilege,
    #[error("not found")]
    NotFound,
}

/// Get the messages of the sessions which the user is a member of,
/// where the time of the message is greater than `end_timestamp`.
///
/// `page_size` is used to limit the number of messages returned in one query,
/// and the messages are returned in descending order of their timestamps.
///
/// # Errors
///
/// Returns `MsgError::DbError` if a database error occurs.
pub async fn get_session_msgs<T: ConnectionTrait>(
    user_id: ID,
    end_timestamp: TimeStamp,
    db_conn: &T,
    page_size: u64,
) -> Result<Paginator<'_, T, sea_orm::SelectModel<user_chat_msg::Model>>, MsgError> {
    let msgs = user_chat_msg::Entity::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT * FROM user_chat_msg
        WHERE time > $1 AND
        EXISTS (SELECT * FROM session_relation WHERE user_id = $2 AND session_id = user_chat_msg.session_id)"#,
                [end_timestamp.into(), user_id.into()],
            ))
            .paginate(db_conn, page_size);
    Ok(msgs)
}

/// Delete a message from the database. The message is specified by `msg_id`.
/// If `owner_id` is `Some`, the function will check whether the deleter is the owner of the
/// session, and return `MsgError::WithoutPrivilege` if not. If `owner_id` is `None`, this check
/// will be skipped.
///
/// Returns `MsgError::NotFound` if the message is not found, or `MsgError::DbError` if a database
/// error occurs.
pub async fn del_msg(
    msg_id: u64,
    owner_id: Option<ID>,
    db_conn: &impl ConnectionTrait,
) -> Result<(), MsgError> {
    let msg_id = msg_id as i64;
    let msg = match UserChatMsg::find_by_id(msg_id).one(db_conn).await? {
        None => return Err(MsgError::NotFound),
        Some(d) => d,
    };
    // TODO:detect whether the deleter is the owner of the session
    if let Some(owner) = owner_id {
        if i64::from(owner) != msg.sender_id {
            return Err(MsgError::WithoutPrivilege);
        }
    }
    msg.delete(db_conn).await?;
    Ok(())
}

/// Insert a new message record into the database.
///
/// The message is specified by `user_id`, `session_id`, `msg`, and `is_encrypted`.
/// The return value is the `MsgID` of the inserted record.
///
/// Returns `MsgError::DbError` if a database error occurs.
pub async fn insert_msg_record(
    user_id: ID,
    session_id: ID,
    msg: serde_json::Value,
    is_encrypted: bool,
    db_conn: &impl ConnectionTrait,
) -> Result<MsgID, MsgError> {
    // TODO:store in binary data
    let msg = user_chat_msg::ActiveModel {
        msg_data: ActiveValue::Set(msg),
        sender_id: ActiveValue::Set(user_id.into()),
        session_id: ActiveValue::Set(session_id.into()),
        is_encrypted: ActiveValue::Set(is_encrypted),
        ..Default::default()
    };
    let msg = msg.insert(db_conn).await?;
    Ok(msg.chat_msg_id.into())
}
