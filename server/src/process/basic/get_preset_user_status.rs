use chrono::Utc;
use pb::service::basic::preset_user_status::v1::{
    GetPresetUserStatusRequest, GetPresetUserStatusResponse,
};
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait, Insert, QueryOrder, QuerySelect};
use tonic::{Request, Response, Status};

use crate::server::BasicServiceProvider;
use entities::{prelude::*, user_status};

#[derive(Debug, thiserror::Error)]
pub enum GetPresetUserStatusErr {
    #[error("Database error")]
    DbError,
}

pub async fn impl_get_preset_user_status(
    server: &BasicServiceProvider,
    _request: Request<GetPresetUserStatusRequest>,
) -> Result<GetPresetUserStatusResponse, GetPresetUserStatusErr> {
    let contents = match UserStatus::find()
        .select_only()
        .column(user_status::Column::Name)
        .order_by_asc(user_status::Column::Time)
        .into_tuple::<String>()
        .all(&server.db.db_pool)
        .await
    {
        Ok(x) => x,
        Err(_) => return Err(GetPresetUserStatusErr::DbError),
    };
    Ok(GetPresetUserStatusResponse { contents })
}

pub async fn get_preset_user_status(
    server: &BasicServiceProvider,
    _request: Request<GetPresetUserStatusRequest>,
) -> Result<Response<GetPresetUserStatusResponse>, Status> {
    match impl_get_preset_user_status(server, _request).await {
        Ok(x) => Ok(Response::new(x)),
        Err(e) => match e {
            GetPresetUserStatusErr::DbError => Err(Status::internal("Database error")),
        },
    }
}

//TODO Move this to server manager
pub async fn add_preset_user_status(dbpool: DatabaseConnection, name: &str) {
    let new_status = user_status::ActiveModel {
        name: ActiveValue::Set(name.to_string()),
        time: ActiveValue::Set(Utc::now().time()),
    };
    Insert::one(new_status)
        .exec(&dbpool)
        .await
        .expect("Failed to add preset user status");
}
