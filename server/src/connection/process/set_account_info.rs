use crate::{
    DbPool,
    client::{
        MsgConvert,
        requests::{self, set_account_info::CHANGE_PUBLIC_TIME},
        response::{ErrorMsgResponse, SetAccountInfoResponse},
    },
    connection::{NetSender, UserInfo},
    consts::ID,
    entities::user,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection};

pub async fn set_account_info(
    user_info: &UserInfo,
    net_sender: impl NetSender,
    request_data: requests::SetAccountRequest,
    db_pool: &DbPool,
) -> anyhow::Result<()> {
    let response = match update_account(user_info.id, request_data, &db_pool.db_pool).await {
        Ok(_) => SetAccountInfoResponse::success().to_msg(),
        Err(SetError::Db(e)) => {
            tracing::error!("Database error: {}", e);
            ErrorMsgResponse::server_error("Database error").to_msg()
        }
        Err(SetError::Type) => {
            tracing::error!("Type error");
            ErrorMsgResponse::server_error("Json format error").to_msg()
        }
        Err(SetError::Unknown(e)) => {
            tracing::error!("Unknown error: {}", e);
            ErrorMsgResponse::server_error("Unknown error").to_msg()
        }
    };
    net_sender
        .send(response)
        .await
        .map_err(|_| anyhow::anyhow!("Failed to send response"))?;
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

async fn update_account(
    id: ID,
    request_data: requests::SetAccountRequest,
    db_conn: &DatabaseConnection,
) -> Result<(), SetError> {
    let mut user = user::ActiveModel {
        id: ActiveValue::Set(id.into()),
        ..Default::default()
    };
    let mut public_updated = false;
    for i in request_data.data {
        if CHANGE_PUBLIC_TIME.contains(&i.0) {
            public_updated = true;
        }
        match i.0 {
            crate::client::basic::SetAccountValues::UserName => {
                if let serde_json::Value::String(name) = i.1 {
                    user.name = ActiveValue::Set(name)
                } else {
                    return Err(SetError::Type);
                }
            }
            crate::client::basic::SetAccountValues::AvatarKey => todo!(),
            crate::client::basic::SetAccountValues::Status => todo!(),
        }
    }
    // update the modified time
    let timestamp = chrono::Utc::now();
    user.update_time = ActiveValue::Set(timestamp.into());
    if public_updated {
        user.public_update_time = ActiveValue::Set(timestamp.into());
    }

    user.update(db_conn).await?;
    Ok(())
}
