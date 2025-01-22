use std::path::PathBuf;

use super::{Files, error_msg::PERMISSION_DENIED, get_id_from_req};
use crate::{
    process::error_msg::SERVER_ERROR,
    server::{DownloadStream, RpcServer},
};
use base::consts::ID;
use base::database::DbPool;
use bytes::BytesMut;
use entities::files;
use pb::ourchat::download::v1::{DownloadRequest, DownloadResponse};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
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

pub async fn check_file_privilege(
    id: ID,
    key: &str,
    db_conn: &DatabaseConnection,
) -> Result<bool, DownloadError> {
    let file = Files::find_by_id(key)
        .filter(files::Column::UserId.eq(id))
        .one(db_conn)
        .await?;
    Ok(file.is_some())
}

async fn download_impl(
    id: ID,
    req: DownloadRequest,
    tx: &mpsc::Sender<Result<DownloadResponse, Status>>,
    db_conn: &DbPool,
    path: impl Into<PathBuf>,
) -> Result<(), DownloadError> {
    // check if the file can be downloaded by the user
    // TODO:check whether it belongs to a session which hold this file
    let path = path.into();
    let ret = check_file_privilege(id, &req.key, &db_conn.db_pool).await?;
    if !ret {
        return Err(DownloadError::PermissionDenied);
    }
    let file = tokio::fs::File::open(&path).await?;
    let mut buf_reader = tokio::io::BufReader::new(file);
    let mut buf = BytesMut::with_capacity(1024);
    loop {
        let n = buf_reader.read_buf(&mut buf).await?;
        if n == 0 {
            break;
        }
        tx.send(Ok(DownloadResponse {
            data: buf.split().freeze(),
        }))
        .await
        .ok();
    }
    Ok(())
}

pub async fn download(
    server: &RpcServer,
    request: Request<DownloadRequest>,
) -> Result<Response<DownloadStream>, Status> {
    let id = get_id_from_req(&request).unwrap();
    let req = request.into_inner();
    let (tx, rx) = mpsc::channel(16);
    let db_conn = server.db.clone();
    let path = server
        .shared_data
        .cfg
        .main_cfg
        .files_storage_path
        .join(&req.key);
    tokio::spawn(async move {
        match download_impl(id, req, &tx, &db_conn, path).await {
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
