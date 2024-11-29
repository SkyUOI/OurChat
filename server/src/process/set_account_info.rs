use crate::{
    component::EmailSender,
    consts::ID,
    entities::user,
    pb::ourchat::set_account_info::v1::{SetSelfInfoRequest, SetSelfInfoResponse},
    server::RpcServer,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection};
use tonic::{Request, Response, Status};

use super::get_id_from_req;

pub async fn set_account_info<T: EmailSender>(
    server: &RpcServer<T>,
    request: Request<SetSelfInfoRequest>,
) -> Result<Response<SetSelfInfoResponse>, Status> {
    let id = get_id_from_req(&request).unwrap();
    let request_data = request.into_inner();
    match update_account(id, request_data, &server.db.db_pool).await {
        Ok(_) => {}
        Err(SetError::Db(e)) => {
            tracing::error!("Database error: {}", e);
            return Err(Status::internal("Database error"));
        }
        Err(SetError::Type) => {
            tracing::error!("Type error");
            return Err(Status::internal("Json format error"));
        }
        Err(SetError::Unknown(e)) => {
            tracing::error!("Unknown error: {}", e);
            return Err(Status::internal("Unknown error"));
        }
    }
    Ok(Response::new(SetSelfInfoResponse {}))
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
    request_data: SetSelfInfoRequest,
    db_conn: &DatabaseConnection,
) -> Result<(), SetError> {
    let mut user = user::ActiveModel {
        id: ActiveValue::Set(id.into()),
        ..Default::default()
    };
    let mut public_updated = false;
    if let Some(name) = request_data.user_name {
        user.name = ActiveValue::Set(name);
        public_updated = true;
    }
    if let Some(status) = request_data.status {
        todo!()
    }
    if let Some(d) = request_data.avatar_key {
        todo!()
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
