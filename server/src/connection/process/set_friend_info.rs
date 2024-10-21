use crate::{
    DbPool,
    client::{MsgConvert, requests::SetFriendInfoRequest, response::SetAccountInfoResponse},
    connection::{NetSender, UserInfo, basic::get_id},
    consts::ID,
};
use derive::db_compatibility;
use sea_orm::{ActiveModelTrait, ActiveValue};

pub async fn set_friend_info(
    user_info: &UserInfo,
    request: SetFriendInfoRequest,
    net_sender: impl NetSender,
    db_pool: &DbPool,
) -> anyhow::Result<()> {
    let ret = match update_friend(user_info.id, request, db_pool).await {
        Ok(_) => SetAccountInfoResponse::success(),
        Err(SetError::Db(e)) => {
            tracing::error!("Database error: {}", e);
            SetAccountInfoResponse::server_error()
        }
        Err(SetError::Type) => {
            tracing::error!("Type format error");
            SetAccountInfoResponse::arg_error()
        }
        Err(SetError::Unknown(e)) => {
            tracing::error!("Unknown error: {}", e);
            SetAccountInfoResponse::server_error()
        }
    };
    net_sender.send(ret.to_msg()).await?;
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

#[db_compatibility]
async fn update_friend(
    id: ID,
    request: SetFriendInfoRequest,
    db_conn: &DbPool,
) -> Result<(), SetError> {
    use entities::friend;
    use entities::prelude::*;

    let mut friend = friend::ActiveModel {
        user_id: ActiveValue::Set(id.into()),
        friend_id: ActiveValue::Set(get_id(&request.ocid, db_conn).await?.into()),
        ..Default::default()
    };
    for i in request.data {
        match i.0 {
            crate::client::basic::SetFriendValues::DisplayName => {
                if let serde_json::Value::String(name) = i.1 {
                    friend.name = ActiveValue::Set(name)
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
