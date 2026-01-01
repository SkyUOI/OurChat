use std::path::PathBuf;

use super::{Files, error_msg::PERMISSION_DENIED};
use crate::{
    process::error_msg::SERVER_ERROR,
    server::{DownloadStream, RpcServer},
};
use base::consts::ID;
use base::database::DbPool;
use bytes::BytesMut;
use entities::session_relation;
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

/// Check if a user is in the specified session
async fn is_user_in_session(
    session_id: i64,
    user_id: ID,
    db: &DbPool,
) -> Result<bool, sea_orm::DbErr> {
    let result = session_relation::Entity::find()
        .filter(session_relation::Column::SessionId.eq(session_id))
        .filter(session_relation::Column::UserId.eq(i64::from(user_id)))
        .one(&db.db_pool)
        .await?;

    Ok(result.is_some())
}

async fn download_impl(
    id: ID,
    req: DownloadRequest,
    tx: &mpsc::Sender<Result<DownloadResponse, Status>>,
    db_conn: &DbPool,
) -> Result<(), DownloadError> {
    // First check if the file exists and get file info
    let file_info = match Files::find_by_id(&req.key).one(&db_conn.db_pool).await? {
        Some(f) => f,
        None => return Err(DownloadError::PermissionDenied),
    };

    // Check if user has permission:
    // 1. User owns the file, OR
    // 2. File has a session_id and user is in that session
    let has_permission = if file_info.user_id == i64::from(id) {
        true // File owner
    } else if let Some(session_id) = file_info.session_id {
        is_user_in_session(session_id, id, db_conn).await? // Session member
    } else {
        false // Neither owner nor in a session
    };

    if !has_permission {
        return Err(DownloadError::PermissionDenied);
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

    tokio::spawn(async move {
        match download_impl(id, req, &tx, &db_conn).await {
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
