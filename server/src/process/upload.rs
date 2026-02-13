use crate::{
    db::file_storage::generate_hierarchical_path,
    helper::{create_file_with_dirs_if_not_exist, generate_random_string},
    process::error_msg::{
        FILE_HASH_ERROR, FILE_SIZE_ERROR, INCORRECT_ORDER, METADATA_ERROR, SERVER_ERROR,
        STORAGE_FULL,
    },
    server::RpcServer,
};
use base::constants::{ID, SessionID};
use entities::{files, prelude::*, user};
use pb::service::ourchat::upload::v1::{UploadRequest, UploadResponse, upload_request};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder,
};
use sha3::{Digest, Sha3_256};
use size::Size;
use std::{num::TryFromIntError, path::PathBuf};
use tokio::{fs, io::AsyncWriteExt};
use tokio_stream::StreamExt;
use tonic::{Response, Status};

/// Configuration for adding a file record
#[derive(Debug)]
pub struct AddFileRecordConfig {
    pub id: ID,
    pub size: Size,
    pub key: String,
    pub auto_clean: bool,
    pub files_storage_path: PathBuf,
    pub limit_size: Size,
    pub session_id: Option<SessionID>,
}

const PREFIX_LEN: usize = 20;

/// Generate a unique key name which refers to the file
/// # Details
/// Generate a 20-character random string, and then add the file's sha256 hash value
/// This ensures uniqueness while maintaining traceability through the hash
fn generate_key_name(hash: impl AsRef<str>) -> String {
    let prefix: String = generate_random_string(PREFIX_LEN);
    format!("{prefix}{}", hash.as_ref())
}

/// Add a new file record to the database without creating the file on disk
///
/// # Arguments
///
/// * `config` - Configuration containing file upload parameters
/// * `db_connection` - Database connection handle
pub async fn add_file_record(
    config: AddFileRecordConfig,
    db_connection: &DatabaseConnection,
) -> Result<(), UploadError> {
    if config.size > config.limit_size {
        return Err(UploadError::FileSizeOverflow);
    }
    let user_info = match User::find_by_id(config.id).one(db_connection).await? {
        Some(user) => user,
        None => Err(anyhow::anyhow!(
            "User {} should exist in database, but not found",
            config.id
        ))?,
    };
    // first check if the limit has been reached
    let res_used = Size::from_bytes(user_info.resource_used);
    let will_used = res_used + config.size;
    tracing::debug!("will used: {}, bytes_num: {}", will_used, config.limit_size);
    if will_used > config.limit_size {
        // reach the limit,delete some files to preserve the limit
        clean_files(will_used - config.limit_size, db_connection, config.id).await?;
    }
    let updated_res_lim = res_used + config.size;
    let mut user_info: user::ActiveModel = user_info.into();
    user_info.resource_used = ActiveValue::Set(updated_res_lim.bytes());
    user_info.update(db_connection).await?;

    let timestamp = chrono::Utc::now().timestamp();

    // Use hierarchical storage path for better filesystem performance
    let hierarchical_path =
        generate_hierarchical_path(&config.files_storage_path, config.id.0, &config.key);
    let path = match hierarchical_path.to_str() {
        Some(path) => path,
        None => {
            return Err(UploadError::InvalidPathError);
        }
    };
    let file = files::ActiveModel {
        key: sea_orm::Set(config.key),
        path: sea_orm::Set(path.to_string()),
        date: sea_orm::Set(timestamp),
        auto_clean: sea_orm::Set(config.auto_clean),
        user_id: sea_orm::Set(config.id.into()),
        session_id: sea_orm::Set(config.session_id.map(|s| s.0 as i64)),
    };
    file.insert(db_connection).await?;
    Ok(())
}

/// Clean up files to free up storage space
/// # Arguments
/// * `need_to_delete` - Amount of space that needs to be freed
/// * `db_connection` - Database connection handle
/// * `user_id` - ID of the user whose files need to be cleaned
///
/// # Details
/// Files are deleted in order of creation date (oldest first) until sufficient space is freed
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

/// Error types that can occur during file upload process
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

/// Internal implementation of the file upload process
/// # Arguments
/// * `server` - Server instance containing shared state
/// * `id` - User ID performing the upload
/// * `request` - Streaming request containing file data
///
/// # Flow
/// 1. Receives metadata in first message
/// 2. Streams content to temporary file while validating size and computing hash
/// 3. Verifies file hash matches expected value
/// 4. Only if verification passes, creates file record and moves file to final location
async fn upload_impl(
    server: &RpcServer,
    id: ID,
    request: tonic::Request<tonic::Streaming<UploadRequest>>,
) -> Result<UploadResponse, UploadError> {
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

    let key = generate_key_name(format!("{:x}", metadata.hash));
    let key_clone = key.clone();
    let files_storage_path = server.shared_data.cfg().main_cfg.files_storage_path.clone();
    let limit_size = server.shared_data.cfg().main_cfg.user_files_limit;
    let session_id = metadata.session_id.map(SessionID);

    // Create temporary file path for streaming using hierarchical structure
    let temp_key = format!("{}.tmp", key);
    let temp_path = generate_hierarchical_path(&files_storage_path, id.0, &temp_key);
    let temp_path_clone = temp_path.clone();

    // Create temporary file and stream content
    let mut temp_file = create_file_with_dirs_if_not_exist(&temp_path).await?;
    let mut hasher = Sha3_256::new();
    let mut sz = 0;
    let logic = async move {
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
            temp_file.write_all(&data).await?;
            hasher.update(&data);
        }

        let hash = hasher.finalize();

        // Verify file size and hash
        if sz != metadata.size as usize {
            tracing::trace!("received size:{}, expected size {}", metadata.size, sz);
            return Err(UploadError::FileSizeError);
        }
        #[allow(deprecated)]
        if hash.as_slice() != metadata.hash {
            tracing::trace!(
                "received hash:{:?}, expected hash {:?}",
                format!("{:x}", metadata.hash),
                format!("{:x}", hash)
            );
            return Err(UploadError::FileHashError);
        }

        // All verifications passed, now create the database record and move file
        let final_path = generate_hierarchical_path(&files_storage_path, id.0, &key);

        // Create database record - this will handle storage limits and cleanup
        let config = AddFileRecordConfig {
            id,
            size: Size::from_bytes(metadata.size),
            key: key.clone(),
            auto_clean: metadata.auto_clean,
            files_storage_path: files_storage_path.clone(),
            limit_size,
            session_id,
        };
        add_file_record(config, &server.db.db_pool).await?;
        temp_file.flush().await?;
        temp_file.sync_all().await?;

        // Move temporary file to final location
        fs::rename(&temp_path, &final_path).await?;
        Ok(())
    };
    match logic.await {
        Ok(_) => {}
        Err(e) => {
            // Clean up temporary file on error
            match fs::remove_file(&temp_path_clone).await {
                Ok(_) => {}
                Err(e) => tracing::error!("Cannot remove temporary file {}", e),
            }
            return Err(e);
        }
    }

    Ok(UploadResponse { key: key_clone })
}

/// Public API endpoint for file uploads
/// Wraps the implementation with proper error handling and status codes
pub async fn upload(
    server: &RpcServer,
    id: ID,
    request: tonic::Request<tonic::Streaming<UploadRequest>>,
) -> Result<Response<UploadResponse>, Status> {
    match upload_impl(server, id, request).await {
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
