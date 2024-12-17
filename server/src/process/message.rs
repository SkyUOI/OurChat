use crate::consts::ID;
use entities::prelude::UserChatMsg;
use sea_orm::{ConnectionTrait, DbErr, EntityTrait, ModelTrait};

pub mod fetch_user_msg;
pub mod recall;
pub mod send_msg;

#[derive(Debug, thiserror::Error)]
enum DelMsgErr {
    #[error("database error:{0}")]
    DbErr(#[from] DbErr),
    #[error("Don't have privilege")]
    WithoutPrivilege,
    #[error("not found")]
    NotFound,
}

async fn del_msg(
    msg_id: u64,
    owner_id: Option<ID>,
    db_conn: &impl ConnectionTrait,
) -> Result<(), DelMsgErr> {
    let msg_id = msg_id as i64;
    let msg = match UserChatMsg::find_by_id(msg_id).one(db_conn).await? {
        None => return Err(DelMsgErr::NotFound),
        Some(d) => d,
    };
    // TODO:detect whether the deleter is the owner of the session
    if let Some(owner) = owner_id {
        if i64::from(owner) != msg.sender_id {
            return Err(DelMsgErr::WithoutPrivilege);
        }
    }
    msg.delete(db_conn).await?;
    Ok(())
}
