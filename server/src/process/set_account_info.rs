use crate::{
    component::EmailSender,
    consts::ID,
    db::{self, file_storage},
    server::RpcServer,
};
use anyhow::Context;
use entities::user;
use pb::ourchat::set_account_info::v1::{SetSelfInfoRequest, SetSelfInfoResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, DbErr, TransactionTrait};
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
        Err(SetError::Unknown(e)) => {
            tracing::error!("Unknown error: {}", e);
            return Err(Status::internal("Unknown error"));
        }
        Err(SetError::Conflict) => {
            return Err(Status::already_exists("Conflict"));
        }
    }
    Ok(Response::new(SetSelfInfoResponse {}))
}

#[derive(Debug, thiserror::Error)]
enum SetError {
    #[error("db error")]
    Db(#[from] DbErr),
    #[error("conflict")]
    Conflict,
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
        user.status = if status.is_empty() {
            ActiveValue::Set(None)
        } else {
            ActiveValue::Set(Some(status))
        };
    }
    let txn = db_conn.begin().await?;
    if let Some(avater_key) = request_data.avatar_key {
        let avater_previous = user.avatar.clone().unwrap();
        let mut should_modified = true;
        // check whether the avatar is different from previous one
        if let Some(avatar) = avater_previous {
            if avatar == avater_key {
                should_modified = false;
            }
            // reduce the refcount
            file_storage::dec_file_refcnt(avatar, &txn)
                .await
                .context("cannot reduce the refcount of file")?;
        } else {
            should_modified = avater_key.is_empty();
        }
        if should_modified {
            user.avatar = ActiveValue::Set(Some(avater_key));
            public_updated = true;
        }
    }
    if let Some(new_ocid) = request_data.ocid {
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
