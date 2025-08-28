use entities::{message_records, prelude::MessageRecords};
use migration::m20241229_022701_add_role_for_session::PredefinedPermissions;
use pb::time::TimeStamp;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ConnectionTrait, DatabaseBackend, EntityTrait, ModelTrait,
    Paginator, PaginatorTrait, Statement,
};

use super::session::if_permission_exist;
use base::consts::{ID, SessionID};
use pb::service::ourchat::msg_delivery::v1::fetch_msgs_response::RespondEventType;

#[derive(Debug, thiserror::Error)]
pub enum MsgError {
    #[error("database error:{0:?}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("unknown error:{0:?}")]
    UnknownError(#[from] anyhow::Error),
    #[error("Don't have privilege")]
    PermissionDenied,
    #[error("not found")]
    NotFound,
}

/// Get the messages of the sessions which the user is a member of,
/// where the time of the message is greater than `end_timestamp`.
///
/// The parameter `page_size` is used to limit the number of messages returned in one query,
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
) -> Result<Paginator<'_, T, sea_orm::SelectModel<message_records::Model>>, MsgError> {
    let msgs = message_records::Entity::find()
        .from_raw_sql(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            r#"SELECT * FROM message_records
WHERE time > $1 AND
((sender_id = $2 OR EXISTS (SELECT * FROM session_relation WHERE user_id = $2 AND session_id = message_records.session_id)) OR (is_all_user = true))"#,
                [end_timestamp.into(), user_id.into()],
            // r#"SELECT * FROM message_records"#,
            // [],
        ))
        .paginate(db_conn, page_size);
    Ok(msgs)
}

/// Delete a message from the database. The message is specified by `msg_id`.
/// If `deleter_id` is `Some`, the function will check whether the deleter has permission to delete the
/// message, and return `MsgError::WithoutPrivilege` if not. If `deleter_id` is `None`, this check
/// will be skipped.
///
/// Returns `MsgError::NotFound` if the message is not found, or `MsgError::DbError` if a database
/// error occurs.
pub async fn del_msg(
    msg_id: u64,
    session_id: SessionID,
    deleter_id: Option<ID>,
    db_conn: &impl ConnectionTrait,
) -> Result<(), MsgError> {
    let msg_id = msg_id as i64;
    let msg = match MessageRecords::find_by_id(msg_id).one(db_conn).await? {
        None => return Err(MsgError::NotFound),
        Some(d) => d,
    };
    if let (Some(owner), Some(sender)) = (deleter_id, msg.sender_id)
        && i64::from(owner) != sender
        && !if_permission_exist(
            owner,
            session_id,
            PredefinedPermissions::RecallMsg.into(),
            db_conn,
        )
        .await?
    {
        return Err(MsgError::PermissionDenied);
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
    sender_id: Option<ID>,
    session_id: Option<SessionID>,
    msg: RespondEventType,
    is_encrypted: bool,
    db_conn: &impl ConnectionTrait,
    is_all_user: bool,
) -> Result<message_records::Model, MsgError> {
    let msg = message_records::ActiveModel {
        msg_data: ActiveValue::Set(serde_json::to_value(msg).unwrap()),
        sender_id: ActiveValue::Set(sender_id.map(i64::from)),
        session_id: ActiveValue::Set(session_id.map(i64::from)),
        is_encrypted: ActiveValue::Set(is_encrypted),
        is_all_user: ActiveValue::Set(is_all_user),
        ..Default::default()
    };
    let msg = msg.insert(db_conn).await?;
    Ok(msg)
}
