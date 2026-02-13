use crate::{process::error_msg::SERVER_ERROR, server::RpcServer};
use base::constants::ID;
use base::database::DbPool;
use pb::service::ourchat::friends::set_friend_info::v1::{
    SetFriendInfoRequest, SetFriendInfoResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DbErr};
use tonic::{Response, Status};

#[derive(Debug, thiserror::Error)]
enum SetError {
    #[error("db error:{0:?}")]
    Db(#[from] DbErr),
    #[error("unknown error:{0:?}")]
    Unknown(#[from] anyhow::Error),
}

pub async fn set_friend_info(
    server: &RpcServer,
    id: ID,
    request: tonic::Request<SetFriendInfoRequest>,
) -> Result<Response<SetFriendInfoResponse>, Status> {
    match set_friend_info_impl(server, id, request).await {
        Ok(d) => Ok(Response::new(d)),
        Err(e) => {
            tracing::error!("{}", e);
            Err(Status::internal(SERVER_ERROR))
        }
    }
}

async fn set_friend_info_impl(
    server: &RpcServer,
    id: ID,
    request: tonic::Request<SetFriendInfoRequest>,
) -> Result<SetFriendInfoResponse, SetError> {
    let request = request.into_inner();
    update_friend(id, request, &server.db).await?;
    Ok(SetFriendInfoResponse {})
}

async fn update_friend(
    id: ID,
    request: SetFriendInfoRequest,
    db_conn: &DbPool,
) -> Result<(), SetError> {
    let mut friend = entities::user_contact_info::ActiveModel {
        user_id: ActiveValue::Set(id.into()),
        contact_user_id: ActiveValue::Set(request.id as i64),
        ..Default::default()
    };
    let mut modified = false;
    if let Some(name) = request.display_name {
        friend.display_name = ActiveValue::Set(Some(name));
        modified = true;
    }
    if !modified {
        return Ok(());
    }
    match friend.clone().update(&db_conn.db_pool).await {
        Ok(_) => {}
        Err(DbErr::RecordNotUpdated) => {
            // record doesn't exist, create it
            match friend.insert(&db_conn.db_pool).await {
                Ok(_) => {}
                Err(e) => return Err(SetError::Db(e)),
            }
        }
        Err(e) => return Err(SetError::Db(e)),
    }
    Ok(())
}
