use pb::service::basic::preset_user_status::v1::{
    GetPresetUserStatusRequest, GetPresetUserStatusResponse,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ConnectionTrait, EntityTrait, QueryOrder, QuerySelect,
};
use tonic::{Request, Response, Status};

use crate::{process::error_msg::SERVER_ERROR, server::BasicServiceProvider};
use entities::{prelude::*, user_status};

#[derive(Debug, thiserror::Error)]
pub enum GetPresetUserStatusErr {
    #[error("Database error:{0:?}")]
    DbError(#[from] sea_orm::DbErr),
}

pub async fn impl_get_preset_user_status(
    server: &BasicServiceProvider,
    _request: Request<GetPresetUserStatusRequest>,
) -> Result<GetPresetUserStatusResponse, GetPresetUserStatusErr> {
    let contents = UserStatus::find()
        .select_only()
        .column(user_status::Column::Name)
        .order_by_asc(user_status::Column::Name)
        .into_tuple::<String>()
        .all(&server.db.db_pool)
        .await?;
    Ok(GetPresetUserStatusResponse { contents })
}

pub async fn get_preset_user_status(
    server: &BasicServiceProvider,
    _request: Request<GetPresetUserStatusRequest>,
) -> Result<Response<GetPresetUserStatusResponse>, Status> {
    match impl_get_preset_user_status(server, _request).await {
        Ok(x) => Ok(Response::new(x)),
        Err(e) => match e {
            GetPresetUserStatusErr::DbError(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
        },
    }
}

// TODO: Move this to server manager
pub async fn add_preset_user_status(dbpool: &impl ConnectionTrait, name: &str) {
    let new_status = user_status::ActiveModel {
        name: ActiveValue::Set(name.to_string()),
    };
    new_status
        .insert(dbpool)
        .await
        .expect("Failed to add preset user status");
}
