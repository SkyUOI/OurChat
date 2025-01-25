use crate::process::get_id_from_req;
use crate::{process::error_msg::SERVER_ERROR, server::RpcServer};
use base::consts::ID;
use base::database::DbPool;
use entities::friend;
use pb::service::ourchat::friends::set_friend_info::v1::{
    SetFriendInfoRequest, SetFriendInfoResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DbErr};
use tonic::{Response, Status};

pub async fn set_friend_info(
    server: &RpcServer,
    request: tonic::Request<SetFriendInfoRequest>,
) -> Result<Response<SetFriendInfoResponse>, Status> {
    let id = get_id_from_req(&request).unwrap();
    let request = request.into_inner();
    match update_friend(id, request, &server.db).await {
        Ok(_) => {}
        Err(e) => {
            return match e {
                SetError::Db(_) | SetError::Unknown(_) => {
                    tracing::error!("{}", e);
                    Err(Status::internal(SERVER_ERROR))
                }
            };
        }
    };
    Ok(Response::new(SetFriendInfoResponse {}))
}

#[derive(Debug, thiserror::Error)]
enum SetError {
    #[error("db error:{0:?}")]
    Db(#[from] DbErr),
    #[error("unknown error:{0:?}")]
    Unknown(#[from] anyhow::Error),
}

async fn update_friend(
    id: ID,
    request: SetFriendInfoRequest,
    db_conn: &DbPool,
) -> Result<(), SetError> {
    let mut friend = friend::ActiveModel {
        user_id: ActiveValue::Set(id.into()),
        friend_id: ActiveValue::Set(request.id as i64),
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
