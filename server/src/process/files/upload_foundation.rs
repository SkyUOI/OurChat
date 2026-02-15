//! Upload session state management for chunked upload API
//!
//! This module provides structures and functions for managing upload sessions
//! using Redis for persistence across server restarts and distributed systems.

use std::{num::TryFromIntError, time::Duration};

use crate::{
    db::redis::redis_key,
    helper::generate_random_string,
    process::error_msg::{
        FILE_HASH_ERROR, FILE_SIZE_ERROR, INCORRECT_ORDER, METADATA_ERROR, SERVER_ERROR,
        STORAGE_FULL,
    },
};
use base::constants::{ID, SessionID};
use pb::time::TimeStampUtc;
use serde::{Deserialize, Serialize};
use size::Size;
use tonic::Status;

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
    #[error("file size error: actual {0} != expected {1}")]
    FileSizeError(usize, usize),
    #[error("invalid path")]
    InvalidPathError,
    #[error("file's size overflows")]
    FileSizeOverflow,
    #[error("redis error:{0:?}")]
    RedisError(#[from] deadpool_redis::redis::RedisError),
    #[error("json error:{0:?}")]
    JsonError(#[from] serde_json::Error),
}

impl From<UploadError> for Status {
    fn from(e: UploadError) -> Self {
        match e {
            UploadError::MetaDataError => Status::invalid_argument(METADATA_ERROR),
            UploadError::Unknown(_)
            | UploadError::DbError(_)
            | UploadError::FromIntError(_)
            | UploadError::InternalIOError(_)
            | UploadError::InvalidPathError
            | UploadError::RedisError(_)
            | UploadError::JsonError(_) => Status::internal(SERVER_ERROR),
            UploadError::StatusError(e) => e,
            UploadError::WrongStructure => Status::invalid_argument(INCORRECT_ORDER),
            UploadError::FileSizeError(..) => Status::invalid_argument(FILE_SIZE_ERROR),
            UploadError::FileHashError => Status::invalid_argument(FILE_HASH_ERROR),
            UploadError::FileSizeOverflow => Status::resource_exhausted(STORAGE_FULL),
        }
    }
}

/// Upload session metadata stored in Redis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadSessionMetadata {
    pub upload_id: String,
    pub user_id: ID,
    pub expected_hash: bytes::Bytes,
    pub expected_size: usize,
    pub auto_clean: bool,
    pub session_id: Option<SessionID>, // Stored as i64, u64 values should be converted
    pub temp_path: String,
    pub bytes_received: usize,
    pub created_at: TimeStampUtc,
    pub last_activity: TimeStampUtc,
}

impl UploadSessionMetadata {
    /// Create new upload session metadata
    pub fn new(
        upload_id: String,
        user_id: ID,
        expected_hash: bytes::Bytes,
        expected_size: usize,
        auto_clean: bool,
        session_id: Option<SessionID>,
        temp_path: std::path::PathBuf,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            upload_id,
            user_id,
            expected_hash,
            expected_size,
            auto_clean,
            session_id,
            temp_path: temp_path.to_string_lossy().to_string(),
            bytes_received: 0,
            created_at: now,
            last_activity: now,
        }
    }

    /// Update last activity timestamp
    pub fn update_activity(&mut self) {
        self.last_activity = chrono::Utc::now();
    }
}

/// Get Redis key for upload session
pub fn upload_session_key(upload_id: &str) -> String {
    redis_key!("upload_session:{}", upload_id)
}

/// Constants for upload timeout
pub const UPLOAD_TIMEOUT_SECONDS: Duration = Duration::from_hours(1);
pub fn recommended_chunk_size() -> Size {
    Size::from_kib(512)
}

const PREFIX_LEN: usize = 20;

/// Generate a unique key name which refers to the file
/// # Details
/// Generate a 20-character random string, and then add the file's sha256 hash value
/// This ensures uniqueness while maintaining traceability through the hash
pub fn generate_key_name(hash: impl AsRef<str>) -> String {
    let prefix: String = generate_random_string(PREFIX_LEN);
    format!("{prefix}{}", hash.as_ref())
}
