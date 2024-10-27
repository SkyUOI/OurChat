use crate::{
    DbPool,
    client::{
        MsgConvert,
        requests::SetFriendInfoRequest,
        response::{ErrorMsgResponse, SetAccountInfoResponse},
    },
    connection::{NetSender, UserInfo, basic::get_id},
    consts::ID,
    entities::{friend, prelude::*},
};
use sea_orm::{ActiveModelTrait, ActiveValue};

pub async fn set_friend_info(
    user_info: &UserInfo,
    request: SetFriendInfoRequest,
    net_sender: impl NetSender,
    db_pool: &DbPool,
) -> anyhow::Result<()> {
    let ret = match update_friend(user_info.id, request, db_pool).await {
        Ok(_) => SetAccountInfoResponse::success().to_msg(),
        Err(SetError::Db(e)) => {
            tracing::error!("Database error: {}", e);
            ErrorMsgResponse::server_error("Database error").to_msg()
        }
        Err(SetError::Type) => {
            tracing::error!("Type format error");
            ErrorMsgResponse::server_error("Json format error").to_msg()
        }
        Err(SetError::Unknown(e)) => {
            tracing::error!("Unknown error: {}", e);
            ErrorMsgResponse::server_error("Unknown error").to_msg()
        }
    };
    net_sender.send(ret).await?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum SetError {
    #[error("db error")]
    Db(#[from] sea_orm::DbErr),
    #[error("type error")]
    Type,
    #[error("unknown error")]
    Unknown(#[from] anyhow::Error),
}

async fn update_friend(
    id: ID,
    request: SetFriendInfoRequest,
    db_conn: &DbPool,
) -> Result<(), SetError> {
    let mut friend = friend::ActiveModel {
        user_id: ActiveValue::Set(id.into()),
        friend_id: ActiveValue::Set(get_id(&request.ocid, db_conn).await?.into()),
        ..Default::default()
    };
    for i in request.data {
        match i.0 {
            crate::client::basic::SetFriendValues::DisplayName => {
                if let serde_json::Value::String(name) = i.1 {
                    friend.display_name = ActiveValue::Set(name)
                } else {
                    tracing::error!("Type error");
                    return Err(SetError::Type);
                }
            }
        }
    }
    friend.update(&db_conn.db_pool).await?;
    Ok(())
}
