use crate::{process::error_msg::SERVER_ERROR, server::ServerManageServiceProvider};
use base::consts::ID;
use entities::user;
use pb::service::server_manage::delete_account::v1::{DeleteAccountRequest, DeleteAccountResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, ConnectionTrait};
use tonic::{Request, Response, Status};
use tracing::info;

#[derive(Debug, thiserror::Error)]
enum DeleteAccountError {
    #[error("database error:{0:?}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("unknown error:{0:?}")]
    UnknownError(#[from] anyhow::Error),
}

/// Set user account status to "deleted"
async fn remove_account(
    id: ID,
    db_connection: &impl ConnectionTrait,
) -> Result<(), DeleteAccountError> {
    let user = user::ActiveModel {
        id: ActiveValue::Set(id.into()),
        ..Default::default()
    };
    user.delete(db_connection).await?;
    info!("user {} deleted by server manage api", id);
    Ok(())
}

async fn delete_account_impl(
    server: &ServerManageServiceProvider,
    request: Request<DeleteAccountRequest>,
) -> Result<DeleteAccountResponse, DeleteAccountError> {
    let req = request.into_inner();
    let id: ID = req.user_id.into();
    let db_conn = &server.db;
    let batch = async {
        remove_account(id, &db_conn.db_pool).await?;
        Ok(())
    };
    match batch.await {
        Ok(_) => Ok(DeleteAccountResponse {}),
        Err(e) => Err(e),
    }
}

pub async fn delete_account(
    server: &ServerManageServiceProvider,
    request: Request<DeleteAccountRequest>,
) -> Result<Response<DeleteAccountResponse>, Status> {
    match delete_account_impl(server, request).await {
        Ok(d) => Ok(Response::new(d)),
        Err(e) => {
            tracing::error!("{}", e);
            Err(Status::internal(SERVER_ERROR))
        }
    }
}
