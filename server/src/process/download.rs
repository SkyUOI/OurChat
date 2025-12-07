use std::path::PathBuf;
use std::sync::Arc;

use super::{Files, error_msg::PERMISSION_DENIED};
use crate::{
    db::file_storage::FileCache,
    process::error_msg::SERVER_ERROR,
    server::{DownloadStream, RpcServer},
};
use base::consts::ID;
use base::database::DbPool;
use bytes::BytesMut;
use entities::files;
use pb::service::ourchat::download::v1::{DownloadRequest, DownloadResponse};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tokio::{io::AsyncReadExt, sync::mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("unknown error:{0:?}")]
    Unknown(#[from] anyhow::Error),
    #[error("database error:{0:?}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("permission denied")]
    PermissionDenied,
    #[error("Internal IO error:{0:?}")]
    InternalIOError(#[from] std::io::Error),
}

async fn download_impl(
    id: ID,
    req: DownloadRequest,
    tx: &mpsc::Sender<Result<DownloadResponse, Status>>,
    db_conn: &DbPool,
    files_storage_path: impl Into<PathBuf>,
    file_cache: Option<Arc<FileCache>>,
) -> Result<(), DownloadError> {
    let _files_storage_path = files_storage_path.into();

    // First check if the file can be downloaded by the user and get file info
    let file_info = match Files::find_by_id(&req.key)
        .filter(files::Column::UserId.eq(id))
        .one(&db_conn.db_pool)
        .await?
    {
        Some(f) => f,
        None => return Err(DownloadError::PermissionDenied),
    };

    // Check cache first if available
    if let Some(cache) = &file_cache
        && let Some(cached_data) = cache.get(&req.key).await
    {
        // Send cached data directly
        tx.send(Ok(DownloadResponse {
            data: cached_data.into(),
        }))
        .await
        .ok();
        return Ok(());
    }

    // Use the stored path from database (which should be hierarchical)
    let path = PathBuf::from(&file_info.path);
    let file = tokio::fs::File::open(&path).await?;
    let mut buf_reader = tokio::io::BufReader::new(file);
    let mut buf = BytesMut::with_capacity(1024);
    let mut file_data = Vec::new();

    loop {
        let n = buf_reader.read_buf(&mut buf).await?;
        if n == 0 {
            break;
        }
        let chunk = buf.split().freeze();
        file_data.extend_from_slice(&chunk);
        tx.send(Ok(DownloadResponse { data: chunk })).await.ok();
    }

    // Add file to cache if cache is available
    if let Some(cache) = &file_cache {
        cache.put(req.key.clone(), file_data).await;
    }

    Ok(())
}

pub async fn download(
    server: &RpcServer,
    id: ID,
    request: Request<DownloadRequest>,
) -> Result<Response<DownloadStream>, Status> {
    let req = request.into_inner();
    let (tx, rx) = mpsc::channel(16);
    let db_conn = server.db.clone();
    let files_storage_path = server.shared_data.cfg.main_cfg.files_storage_path.clone();
    let file_cache = server
        .shared_data
        .file_sys
        .as_ref()
        .map(|fs| fs.get_cache());
    tokio::spawn(async move {
        match download_impl(id, req, &tx, &db_conn, files_storage_path, file_cache).await {
            Ok(_) => {}
            Err(e) => match e {
                DownloadError::PermissionDenied => {
                    tx.send(Err(Status::permission_denied(PERMISSION_DENIED)))
                        .await
                        .ok();
                }
                _ => {
                    tracing::error!("{}", e);
                    tx.send(Err(Status::internal(SERVER_ERROR))).await.ok();
                }
            },
        }
    });
    let output_stream = ReceiverStream::new(rx);
    Ok(Response::new(Box::pin(output_stream) as DownloadStream))
}
