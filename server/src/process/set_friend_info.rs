use super::{basic::get_id, get_id_from_req};
use crate::{
    DbPool,
    component::EmailSender,
    consts::ID,
    entities::friend,
    pb::ourchat::set_account_info::v1::{SetFriendInfoRequest, SetFriendInfoResponse},
    server::RpcServer,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DbErr};
use tonic::{Response, Status};

pub async fn set_friend_info<T: EmailSender>(
    server: &RpcServer<T>,
    request: tonic::Request<SetFriendInfoRequest>,
) -> Result<Response<SetFriendInfoResponse>, tonic::Status> {
    let id = get_id_from_req(&request).unwrap();
    let request = request.into_inner();
    match update_friend(id, request, &server.db).await {
        Ok(_) => {}
        Err(SetError::Db(e)) => {
            tracing::error!("Database error: {}", e);
            return Err(Status::internal("Database error"));
        }
        Err(SetError::Unknown(e)) => {
            tracing::error!("Unknown error: {}", e);
            return Err(Status::internal("Unknown error"));
        }
    };
    Ok(Response::new(SetFriendInfoResponse {}))
}

#[derive(Debug, thiserror::Error)]
enum SetError {
    #[error("db error")]
    Db(#[from] sea_orm::DbErr),
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
    let mut modified = false;
    if let Some(name) = request.display_name {
        friend.display_name = ActiveValue::Set(name);
        modified = true;
    }
    if !modified {
        return Ok(());
    }
    match friend.clone().update(&db_conn.db_pool).await {
        Ok(_) => {}
        Err(DbErr::RecordNotUpdated) => {
            // record not existed, create it
            match friend.insert(&db_conn.db_pool).await {
                Ok(_) => {}
                Err(e) => return Err(SetError::Db(e)),
            }
        }
        Err(e) => return Err(SetError::Db(e)),
    }
    Ok(())
}
