use super::{
    error_msg::{
        CONFLICT,
        invalid::{self, OCID_TOO_LONG, STATUS_TOO_LONG},
    },
    mapped_to_user_defined_status,
};
use crate::{
    db::{self, file_storage},
    process::error_msg::SERVER_ERROR,
    server::RpcServer,
};
use anyhow::{Context, anyhow};
use base::constants::ID;
use chrono::Duration;
use deadpool_redis::redis::AsyncCommands;
use entities::user;
use migration::constants::{OCID_MAX_LEN, USERNAME_MAX_LEN};
use pb::service::ourchat::set_account_info::v1::{SetSelfInfoRequest, SetSelfInfoResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DbErr, EntityTrait, TransactionTrait};
use tonic::{Request, Response, Status};

pub const STATUS_LENGTH_MAX: usize = 128;

pub async fn set_self_info(
    server: &RpcServer,
    id: ID,
    request: Request<SetSelfInfoRequest>,
) -> Result<Response<SetSelfInfoResponse>, Status> {
    let request_data = request.into_inner();
    match update_account(server, id, request_data).await {
        Ok(_) => Ok(Response::new(SetSelfInfoResponse {})),
        Err(e) => match e {
            SetError::Db(_) | SetError::Unknown(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            SetError::Conflict => Err(Status::already_exists(CONFLICT)),
            SetError::Status(s) => Err(s),
        },
    }
}

#[derive(Debug, thiserror::Error)]
enum SetError {
    #[error("db error:{0:?}")]
    Db(#[from] DbErr),
    #[error("conflict")]
    Conflict,
    #[error("unknown error:{0:?}")]
    Unknown(#[from] anyhow::Error),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
}

async fn update_account(
    server: &RpcServer,
    id: ID,
    request_data: SetSelfInfoRequest,
) -> Result<(), SetError> {
    // Check username length
    if let Some(name) = &request_data.user_name
        && (name.len() > USERNAME_MAX_LEN || name.trim().is_empty())
    {
        Err(Status::invalid_argument(invalid::USERNAME))?
    }

    // Check status length
    if let Some(status) = &request_data.user_defined_status
        && status.len() > STATUS_LENGTH_MAX
    {
        Err(Status::invalid_argument(STATUS_TOO_LONG))?
    }
    let expire_time = server
        .shared_data
        .cfg()
        .main_cfg
        .user_defined_status_expire_time;
    let user_defined_status_expire_time =
        Duration::from_std(expire_time).context("Could convert expire_time to chrono::Duration")?;
    let original_user = user::Entity::find_by_id(id)
        .one(&server.db.db_pool)
        .await?
        .ok_or(anyhow!("user not found"))?;
    let mut user = user::ActiveModel {
        id: ActiveValue::Set(id.into()),
        ..Default::default()
    };
    let mut public_updated = false;
    if let Some(name) = request_data.user_name
        && name != original_user.name
    {
        user.name = ActiveValue::Set(name);
        public_updated = true;
    }
    let mut redis_conn = server.db.get_redis_connection().await?;
    if let Some(status) = request_data.user_defined_status {
        let key = mapped_to_user_defined_status(user.id.as_ref());
        let _: () = redis_conn
            .set_ex(
                key,
                status,
                user_defined_status_expire_time.num_seconds() as u64,
            )
            .await
            .context("Cannot set user defined status to redis")?;
        public_updated = true;
    }
    let txn = server.db.db_pool.begin().await?;
    if let Some(avatar_key) = request_data.avatar_key
        && Some(&avatar_key) != original_user.avatar.as_ref()
    {
        // Simple ownership model: delete old avatar file if it exists
        if let Some(old_avatar_key) = original_user.avatar {
            // Delete the old avatar file - no reference counting needed
            if let Err(e) = file_storage::delete_file(&old_avatar_key, &txn).await {
                tracing::warn!("Failed to delete old avatar file {}: {}", old_avatar_key, e);
                // Continue anyway - the file will be cleaned up by auto-cleanup later
            }
        }

        user.avatar = ActiveValue::Set(Some(avatar_key));
        public_updated = true;
    }
    if let Some(new_ocid) = request_data.ocid {
        if new_ocid.len() > OCID_MAX_LEN {
            return Err(SetError::Status(Status::invalid_argument(OCID_TOO_LONG)));
        }
        user.ocid = ActiveValue::Set(new_ocid);
        public_updated = true;
    }
    // update the modified time
    let timestamp = chrono::Utc::now();
    user.update_time = ActiveValue::Set(timestamp.into());
    if public_updated {
        user.public_update_time = ActiveValue::Set(timestamp.into());
    }
    match user.update(&txn).await {
        Ok(_) => {}
        Err(e) => {
            if db::helper::is_conflict(&e) {
                return Err(SetError::Conflict);
            }
            return Err(SetError::Db(e));
        }
    }
    txn.commit().await?;
    Ok(())
}
