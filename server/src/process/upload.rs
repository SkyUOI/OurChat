use super::get_id_from_req;
use crate::{
    process::error_msg::{
        FILE_HASH_ERROR, FILE_SIZE_ERROR, INCORRECT_ORDER, METADATA_ERROR, SERVER_ERROR,
        STORAGE_FULL, not_found,
    },
    server::RpcServer,
    utils::{create_file_with_dirs_if_not_exist, generate_random_string},
};
use base::consts::ID;
use entities::{files, prelude::*, user};
use pb::ourchat::upload::v1::{UploadRequest, UploadResponse, upload_request};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder,
};
use sha3::{Digest, Sha3_256};
use size::Size;
use std::{num::TryFromIntError, path::PathBuf};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use tokio_stream::StreamExt;
use tonic::{Response, Status};

const PREFIX_LEN: usize = 20;

/// Generate a unique key name which refers to the file
/// # Details
/// Generate a 20-character random string, and then add the file's sha256 hash value
fn generate_key_name(hash: &str) -> String {
    let prefix: String = generate_random_string(PREFIX_LEN);
    format!("{}{}", prefix, hash)
}

pub async fn add_file_record(
    id: ID,
    sz: Size,
    key: String,
    auto_clean: bool,
    db_connection: &DatabaseConnection,
    files_storage_path: impl Into<PathBuf>,
    limit_size: Size,
) -> Result<File, UploadError> {
    if sz > limit_size {
        return Err(UploadError::FileSizeOverflow);
    }
    let user_info = match User::find_by_id(id).one(db_connection).await? {
        Some(user) => user,
        None => Err(anyhow::anyhow!(
            "User {} should exist in database, but not found",
            id
        ))?,
    };
    // first check if the limit has been reached
    let res_used = Size::from_bytes(user_info.resource_used);
    let will_used = res_used + sz;
    tracing::debug!("will used: {}, bytes_num: {}", will_used, limit_size);
    if will_used > limit_size {
        // reach the limit,delete some files to preserve the limit
        clean_files(will_used - limit_size, db_connection, id).await?;
    }
    let updated_res_lim = res_used + sz;
    let mut user_info: user::ActiveModel = user_info.into();
    user_info.resource_used = ActiveValue::Set(updated_res_lim.bytes());
    user_info.update(db_connection).await?;

    let timestamp = chrono::Utc::now().timestamp();
    let path: PathBuf = files_storage_path.into();
    let path = path.join(&key);
    let path = match path.to_str() {
        Some(path) => path,
        None => {
            return Err(UploadError::InvalidPathError);
        }
    };
    let file = files::ActiveModel {
        key: sea_orm::Set(key),
        path: sea_orm::Set(path.to_string()),
        date: sea_orm::Set(timestamp),
        auto_clean: sea_orm::Set(auto_clean),
        user_id: sea_orm::Set(id.into()),
        ref_cnt: sea_orm::Set(1),
    };
    file.insert(db_connection).await?;
    let f = create_file_with_dirs_if_not_exist(&path).await?;
    Ok(f)
}

pub async fn clean_files(
    need_to_delete: Size,
    db_connection: &impl ConnectionTrait,
    user_id: ID,
) -> Result<(), UploadError> {
    tracing::debug!("Begin to clean files");
    let mut files_pages = Files::find()
        .filter(files::Column::UserId.eq(user_id.0))
        .order_by_asc(files::Column::Date)
        .paginate(db_connection, 150);
    let mut deleted_size = Size::from_bytes(0);
    'reserve_space: while let Some(files) = files_pages.fetch_and_next().await? {
        for file in files {
            let delta_size = Size::from_bytes(fs::metadata(&file.path).await?.len());
            deleted_size += delta_size;
            fs::remove_file(&file.path).await?;
            tracing::debug!("Deleted file: {}", &file.key);
            Files::delete_by_id(&file.key).exec(db_connection).await?;
            if deleted_size + delta_size > need_to_delete {
                break 'reserve_space;
            }
        }
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum UploadError {
    #[error("Metadata error")]
    MetaDataError,
    #[error("unknown error:{0:?}")]
    Unknown(#[from] anyhow::Error),
    #[error("status:{0:?}")]
    StatusError(#[from] Status),
    #[error("database error:{0:?}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("from int error")]
    FromIntError(#[from] TryFromIntError),
    #[error("Internal IO error:{0:?}")]
    InternalIOError(#[from] std::io::Error),
    #[error("wrong structure")]
    WrongStructure,
    #[error("file hash error")]
    FileHashError,
    #[error("file size error")]
    FileSizeError,
    #[error("invalid path")]
    InvalidPathError,
    #[error("file's size overflows")]
    FileSizeOverflow,
}

async fn upload_impl(
    server: &RpcServer,
    request: tonic::Request<tonic::Streaming<UploadRequest>>,
) -> Result<UploadResponse, UploadError> {
    let id = get_id_from_req(&request).unwrap();
    let mut stream_req = request.into_inner();
    let metadata = match stream_req.next().await {
        None => {
            return Err(UploadError::MetaDataError);
        }
        Some(meta) => meta?,
    };
    let metadata = match metadata.header() {
        None => {
            return Err(UploadError::MetaDataError);
        }
        Some(data) => data,
    };
    let key = generate_key_name(&metadata.hash);
    let files_storage_path = &server.shared_data.cfg.main_cfg.files_storage_path;
    let limit_size = server.shared_data.cfg.main_cfg.user_files_limit;
    let mut file_handle = add_file_record(
        id,
        Size::from_bytes(metadata.size),
        key.clone(),
        metadata.auto_clean,
        &server.db.db_pool,
        &files_storage_path,
        limit_size,
    )
    .await?;
    let mut hasher = Sha3_256::new();
    let mut sz = 0;
    while let Some(data) = stream_req.next().await {
        let data = match data?.data {
            Some(upload_request::Data::Content(data)) => data,
            _ => {
                return Err(UploadError::WrongStructure);
            }
        };
        sz += data.len();
        if sz > metadata.size as usize {
            return Err(UploadError::FileSizeError);
        }
        file_handle.write_all(&data).await?;
        hasher.update(&data);
    }
    let hash = hasher.finalize();
    if sz != metadata.size as usize {
        tracing::trace!("received size:{}, expected size {}", metadata.size, sz);
        return Err(UploadError::FileSizeError);
    }
    if format!("{:x}", hash) != metadata.hash {
        tracing::trace!(
            "received hash:{:?}, expected hash {:?}",
            metadata.hash,
            format!("{:x}", hash)
        );
        return Err(UploadError::FileHashError);
    }
    Ok(UploadResponse { key })
}

pub async fn upload(
    server: &RpcServer,
    request: tonic::Request<tonic::Streaming<UploadRequest>>,
) -> Result<Response<UploadResponse>, Status> {
    match upload_impl(server, request).await {
        Ok(ok_resp) => Ok(Response::new(ok_resp)),
        Err(e) => {
            tracing::error!("{}", e);
            match e {
                UploadError::MetaDataError => Err(Status::invalid_argument(METADATA_ERROR)),
                UploadError::Unknown(_)
                | UploadError::DbError(_)
                | UploadError::FromIntError(_)
                | UploadError::InternalIOError(_)
                | UploadError::InvalidPathError => {
                    tracing::error!("{}", e);
                    Err(Status::internal(SERVER_ERROR))
                }
                UploadError::StatusError(e) => Err(e),
                UploadError::WrongStructure => Err(Status::invalid_argument(INCORRECT_ORDER)),
                UploadError::FileSizeError => Err(Status::invalid_argument(FILE_SIZE_ERROR)),
                UploadError::FileHashError => Err(Status::invalid_argument(FILE_HASH_ERROR)),
                UploadError::FileSizeOverflow => Err(Status::resource_exhausted(STORAGE_FULL)),
            }
        }
    }
}
