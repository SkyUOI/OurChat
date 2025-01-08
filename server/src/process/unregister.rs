use super::get_id_from_req;
use crate::server::RpcServer;
use base::consts::ID;
use entities::user;
use pb::ourchat::unregister::v1::{UnregisterRequest, UnregisterResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, ConnectionTrait};
use tonic::{Response, Status};

#[derive(Debug, thiserror::Error)]
enum UnregisterError {
    #[error("database error:{0:?}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("unknown error:{0:?}")]
    UnknownError(#[from] anyhow::Error),
}

/// Remove user from database
async fn remove_account(
    id: ID,
    db_connection: &impl ConnectionTrait,
) -> Result<(), UnregisterError> {
    let user = user::ActiveModel {
        id: ActiveValue::Set(id.into()),
        ..Default::default()
    };
    user.delete(db_connection).await?;
    Ok(())
}

async fn unregister_impl(
    server: &RpcServer,
    request: tonic::Request<UnregisterRequest>,
) -> Result<UnregisterResponse, UnregisterError> {
    let db_conn = &server.db;
    let id = get_id_from_req(&request).unwrap();
    let batch = async {
        remove_account(id, &db_conn.db_pool).await?;
        Ok(())
    };
    match batch.await {
        Ok(_) => Ok(UnregisterResponse {}),
        Err(e) => Err(e),
    }
}

pub async fn unregister(
    server: &RpcServer,
    request: tonic::Request<UnregisterRequest>,
) -> Result<Response<UnregisterResponse>, Status> {
    match unregister_impl(server, request).await {
        Ok(d) => Ok(Response::new(d)),
        Err(e) => {
            tracing::error!("{}", e);
            Err(Status::internal("Server Error"))
        }
    }
}
