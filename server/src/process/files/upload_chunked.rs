use std::sync::Arc;

use base::constants::{ID, SessionID};
use deadpool_redis::redis::AsyncCommands;
use pb::service::ourchat::upload::v1::{
    CancelUploadRequest, CancelUploadResponse, CompleteUploadRequest, CompleteUploadResponse,
    StartUploadRequest, StartUploadResponse, UploadChunkRequest, UploadChunkResponse,
};
use sha3::{Digest, Sha3_256};
use size::Size;
use tokio::{
    fs::{self, create_dir_all},
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
    select,
    sync::Notify,
};
use tokio_stream::{StreamExt, wrappers::ReadDirStream};
use tonic::{Response, Status};
use tracing::info;

use crate::{
    db::file_storage::generate_hierarchical_path,
    helper::{create_file_with_dirs_if_not_exist, generate_random_string},
    process::{
        LocalUploadState,
        error_msg::{self, SERVER_ERROR},
        files::{
            upload::{AddFileRecordConfig, add_file_record},
            upload_foundation::{
                UPLOAD_TIMEOUT_SECONDS, UploadError, UploadSessionMetadata, generate_key_name,
                recommended_chunk_size, upload_session_key,
            },
        },
    },
    server::RpcServer,
};

/// Helper: Save upload session metadata to Redis
async fn save_session_to_redis(
    redis: &mut deadpool_redis::Connection,
    metadata: &UploadSessionMetadata,
) -> Result<(), UploadError> {
    let key = upload_session_key(&metadata.upload_id);
    let value = serde_json::to_string(metadata)?;
    let _: () = redis
        .set_ex(&key, value, UPLOAD_TIMEOUT_SECONDS.as_secs())
        .await?;
    Ok(())
}

/// Helper: Get upload session metadata from Redis
async fn get_session_from_redis(
    redis: &mut deadpool_redis::Connection,
    upload_id: &str,
) -> Result<Option<UploadSessionMetadata>, UploadError> {
    let key = upload_session_key(upload_id);
    let value: Option<String> = redis.get(&key).await?;
    match value {
        Some(v) => {
            let metadata: UploadSessionMetadata = serde_json::from_str(&v)?;
            Ok(Some(metadata))
        }
        None => Ok(None),
    }
}

/// Helper: Delete upload session metadata from Redis
async fn delete_session_from_redis(
    redis: &mut deadpool_redis::Connection,
    upload_id: &str,
) -> Result<(), UploadError> {
    let key = upload_session_key(upload_id);
    let _: () = redis.del(&key).await?;
    Ok(())
}

/// Start a new chunked upload session
pub async fn start_upload_impl(
    server: &RpcServer,
    id: ID,
    request: tonic::Request<StartUploadRequest>,
) -> Result<StartUploadResponse, UploadError> {
    let req = request.into_inner();
    let mut redis = server.db.get_redis_connection().await?;

    // Check user quota and size limits
    let limit_size = server.shared_data.cfg().main_cfg.user_files_limit;
    if req.size > limit_size.bytes() as u64 {
        return Err(UploadError::FileSizeOverflow);
    }

    // Generate unique upload ID
    let id_itself = generate_random_string(32);
    let upload_id = format!("upload_{}_{}", id, id_itself);

    // Generate temporary file path
    let files_storage_path = server.shared_data.cfg().main_cfg.files_storage_path.clone();
    let temp_path = generate_hierarchical_path(&files_storage_path, id, &id_itself);
    info!(
        "Generated temp path for upload {}: {}",
        upload_id,
        temp_path.display()
    );

    // Create temp dir
    create_dir_all(&temp_path).await?;

    let session_id = req.session_id.map(SessionID);
    // Create metadata
    let metadata = UploadSessionMetadata::new(
        upload_id.clone(),
        id,
        req.hash,
        req.size as usize,
        req.auto_clean,
        session_id,
        temp_path.clone(),
    );

    // Save to Redis with TTL
    save_session_to_redis(&mut redis, &metadata).await?;

    let cleanup_notifier = Arc::new(Notify::new());
    // Store file handle and hasher in local memory
    let local_state = LocalUploadState {
        temp_dir: temp_path.clone(),
        cleanup: cleanup_notifier.clone(),
    };
    server
        .shared_data
        .upload_local_state
        .insert(upload_id.clone(), local_state);

    // Spawn per-upload cleanup task for local resources
    let upload_id_clone = upload_id.clone();
    let local_state_clone = server.shared_data.upload_local_state.clone();
    tokio::spawn(async move {
        select! {
            _ = cleanup_notifier.notified() => {
                tracing::info!("Received cleanup notification for upload {}", upload_id_clone);
            }
            _ = tokio::time::sleep(UPLOAD_TIMEOUT_SECONDS) => {
                tracing::info!("Upload {} timed out after {:?}", upload_id_clone, UPLOAD_TIMEOUT_SECONDS);
            }
        }
        if let Some((_, state)) = local_state_clone.remove(&upload_id_clone) {
            match fs::remove_dir_all(&state.temp_dir).await {
                Ok(_) => tracing::info!("Cleaned up upload {}", upload_id_clone),
                Err(e) => tracing::error!("Failed to clean up upload {}: {:?}", upload_id_clone, e),
            }
        }
    });

    Ok(StartUploadResponse {
        upload_id,
        chunk_size: recommended_chunk_size().bytes() as u32,
        timeout_seconds: UPLOAD_TIMEOUT_SECONDS.as_secs() as u32,
    })
}

pub async fn start_upload(
    server: &RpcServer,
    id: ID,
    request: tonic::Request<StartUploadRequest>,
) -> Result<Response<StartUploadResponse>, Status> {
    match start_upload_impl(server, id, request).await {
        Ok(ok_resp) => Ok(Response::new(ok_resp)),
        Err(e) => {
            tracing::error!("{}", e);
            Err(e.into())
        }
    }
}

/// Upload a chunk of data
pub async fn upload_chunk_impl(
    server: &RpcServer,
    id: ID,
    request: tonic::Request<UploadChunkRequest>,
) -> Result<UploadChunkResponse, UploadError> {
    let req = request.into_inner();
    let mut redis = server.db.get_redis_connection().await?;

    // Get metadata from Redis
    let mut metadata = match get_session_from_redis(&mut redis, &req.upload_id).await? {
        Some(m) => m,
        None => Err(Status::not_found(
            crate::process::error_msg::not_found::UPLOAD_SESSION,
        ))?,
    };

    // Verify user ownership
    if metadata.user_id != id {
        Err(Status::permission_denied(
            crate::process::error_msg::PERMISSION_DENIED,
        ))?
    }

    // Check if local state exists (file must be on this instance)
    let local_state = match server.shared_data.upload_local_state.get(&req.upload_id) {
        Some(state) => state,
        None => Err(Status::failed_precondition(
            error_msg::UPLOAD_SESSION_NOT_IN_THIS_INSTANCE,
        ))?,
    };

    // Write chunk to file
    {
        let mut file =
            create_file_with_dirs_if_not_exist(local_state.temp_dir.join(req.chunk_id.to_string()))
                .await?;
        file.write_all(&req.chunk_data).await?;
    }

    // Update metadata
    metadata.bytes_received += req.chunk_data.len();
    metadata.update_activity();

    // Check if size exceeded
    if metadata.bytes_received > metadata.expected_size {
        // Clean up
        let _ = delete_session_from_redis(&mut redis, &req.upload_id).await;
        local_state.cleanup();
        Err(UploadError::FileSizeError(
            metadata.bytes_received,
            metadata.expected_size,
        ))?
    }

    // Save updated metadata to Redis (refreshes TTL)
    save_session_to_redis(&mut redis, &metadata).await?;

    Ok(UploadChunkResponse {
        bytes_received: metadata.bytes_received as u64,
    })
}

pub async fn upload_chunk(
    server: &RpcServer,
    id: ID,
    request: tonic::Request<UploadChunkRequest>,
) -> Result<Response<UploadChunkResponse>, Status> {
    match upload_chunk_impl(server, id, request).await {
        Ok(ok_resp) => Ok(Response::new(ok_resp)),
        Err(e) => {
            tracing::error!("{}", e);
            Err(e.into())
        }
    }
}

/// Complete an upload
pub async fn complete_upload_impl(
    server: &RpcServer,
    id: ID,
    request: tonic::Request<CompleteUploadRequest>,
) -> Result<CompleteUploadResponse, UploadError> {
    let req = request.into_inner();
    let mut redis = server.db.get_redis_connection().await?;

    // Get metadata from Redis
    let metadata = match get_session_from_redis(&mut redis, &req.upload_id).await? {
        Some(m) => m,
        None => Err(Status::not_found(
            crate::process::error_msg::not_found::UPLOAD_SESSION,
        ))?,
    };

    // Verify user ownership
    if metadata.user_id != id {
        Err(Status::permission_denied(
            crate::process::error_msg::PERMISSION_DENIED,
        ))?
    }

    // Delete from Redis
    delete_session_from_redis(&mut redis, &req.upload_id).await?;

    // Get local state
    let local_state = match server.shared_data.upload_local_state.remove(&req.upload_id) {
        Some((_key, state)) => state,
        None => Err(Status::internal(SERVER_ERROR))?,
    };
    // Verify size
    if metadata.bytes_received != metadata.expected_size {
        local_state.cleanup();
        return Err(UploadError::FileSizeError(
            metadata.bytes_received,
            metadata.expected_size,
        ));
    }

    // Generate final key
    let hash_hex = format!("{:x}", metadata.expected_hash);
    let key = generate_key_name(&hash_hex);
    let final_path = generate_hierarchical_path(
        &server.shared_data.cfg().main_cfg.files_storage_path,
        id,
        &key,
    );
    // write file
    let mut file = create_file_with_dirs_if_not_exist(&final_path).await?;
    let mut tmp_files: Vec<_> = ReadDirStream::new(fs::read_dir(&local_state.temp_dir).await?)
        .filter_map(|item| {
            item.ok()
                .and_then(|x| x.path().is_file().then(|| x.file_name()))
        })
        .filter_map(|item| item.to_string_lossy().to_string().parse::<usize>().ok())
        .collect()
        .await;
    tmp_files.sort_unstable();
    let mut hasher = Sha3_256::new();
    let mut buf = vec![0u8; 1024];
    for i in tmp_files {
        let chunk_path = local_state.temp_dir.join(i.to_string());
        let mut chunk_file = fs::File::open(&chunk_path).await?;
        tokio::io::copy(&mut chunk_file, &mut file).await?;
        chunk_file.seek(std::io::SeekFrom::Start(0)).await?;
        while let Ok(n) = chunk_file.read(&mut buf).await {
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
    }
    let actual_hash = hasher.finalize();
    if actual_hash.as_slice() != metadata.expected_hash {
        // Clean up file and local state
        let _ = fs::remove_file(&final_path).await;
        local_state.cleanup();
        return Err(UploadError::FileHashError);
    }

    // Create database record
    let config = AddFileRecordConfig {
        id,
        size: Size::from_bytes(metadata.expected_size),
        key: key.clone(),
        auto_clean: metadata.auto_clean,
        files_storage_path: server.shared_data.cfg().main_cfg.files_storage_path.clone(),
        limit_size: server.shared_data.cfg().main_cfg.user_files_limit,
        session_id: metadata.session_id,
    };
    add_file_record(config, &server.db.db_pool).await?;

    Ok(CompleteUploadResponse { key })
}

pub async fn complete_upload(
    server: &RpcServer,
    id: ID,
    request: tonic::Request<CompleteUploadRequest>,
) -> Result<Response<CompleteUploadResponse>, Status> {
    match complete_upload_impl(server, id, request).await {
        Ok(ok_resp) => Ok(Response::new(ok_resp)),
        Err(e) => {
            tracing::error!("{}", e);
            Err(e.into())
        }
    }
}

/// Cancel an upload
pub async fn cancel_upload_impl(
    server: &RpcServer,
    id: ID,
    request: tonic::Request<CancelUploadRequest>,
) -> Result<CancelUploadResponse, UploadError> {
    let req = request.into_inner();
    let mut redis = server.db.get_redis_connection().await?;

    // Get metadata from Redis (if exists)
    if let Ok(Some(metadata)) = get_session_from_redis(&mut redis, &req.upload_id).await {
        // Verify user ownership
        if metadata.user_id != id {
            Err(Status::permission_denied(
                crate::process::error_msg::PERMISSION_DENIED,
            ))?
        }

        // Delete from Redis
        let _ = delete_session_from_redis(&mut redis, &req.upload_id).await;

        // Clean up local state if present
        if let Some((_, local_state)) = server.shared_data.upload_local_state.remove(&req.upload_id)
        {
            local_state.cleanup();
        }
    } else {
        Err(Status::not_found(
            crate::process::error_msg::not_found::UPLOAD_SESSION,
        ))?
    }

    Ok(CancelUploadResponse {})
}

pub async fn cancel_upload(
    server: &RpcServer,
    id: ID,
    request: tonic::Request<CancelUploadRequest>,
) -> Result<Response<CancelUploadResponse>, Status> {
    match cancel_upload_impl(server, id, request).await {
        Ok(ok_resp) => Ok(Response::new(ok_resp)),
        Err(e) => {
            tracing::error!("{}", e);
            Err(e.into())
        }
    }
}
