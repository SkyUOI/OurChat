use crate::{process::error_msg::SERVER_ERROR, server::RpcServer};
use base::consts::ID;
use entities::user;
use migration::m20250301_005919_add_soft_delete_columns::AccountStatus;
use pb::service::ourchat::unregister::v1::{UnregisterRequest, UnregisterResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, ConnectionTrait};
use tonic::{Request, Response, Status};

#[derive(Debug, thiserror::Error)]
enum UnregisterError {
    #[error("database error:{0:?}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("unknown error:{0:?}")]
    UnknownError(#[from] anyhow::Error),
}

/// Set user account status to deleted
async fn set_account_deleted(
    id: ID,
    db_connection: &impl ConnectionTrait,
) -> Result<(), UnregisterError> {
    let user = user::ActiveModel {
        id: ActiveValue::Set(id.into()),
        account_status: ActiveValue::Set(AccountStatus::Deleted.into()), // Deleted
        ..Default::default()
    };
    user.update(db_connection).await?;
    Ok(())
}

/// totoally delete user account and all related data
async fn delete_account(
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
    id: ID,
    _request: Request<UnregisterRequest>,
) -> Result<UnregisterResponse, UnregisterError> {
    let db_conn = &server.db;
    let batch = async {
        match server.shared_data.cfg.main_cfg.unregister_policy {
            crate::config::UnregisterPolicy::Disable => {
                set_account_deleted(id, &db_conn.db_pool).await?;
            }
            crate::config::UnregisterPolicy::Delete => {
                delete_account(id, &db_conn.db_pool).await?;
            }
        }
        Ok(())
    };
    match batch.await {
        Ok(_) => Ok(UnregisterResponse {}),
        Err(e) => Err(e),
    }
}

pub async fn unregister(
    server: &RpcServer,
    id: ID,
    request: Request<UnregisterRequest>,
) -> Result<Response<UnregisterResponse>, Status> {
    match unregister_impl(server, id, request).await {
        Ok(d) => Ok(Response::new(d)),
        Err(e) => {
            tracing::error!("{}", e);
            Err(Status::internal(SERVER_ERROR))
        }
    }
}
