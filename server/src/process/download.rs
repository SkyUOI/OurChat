use crate::{
    DbPool,
    component::EmailSender,
    consts::ID,
    entities::files,
    pb,
    server::{DownloadStream, RpcServer},
};
use bytes::BytesMut;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tokio::{io::AsyncReadExt, sync::mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use super::{Files, get_id_from_req};

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("unknown error:{0}")]
    Unknown(#[from] anyhow::Error),
    #[error("database error:{0}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("permission denied")]
    PermissionDenied,
    #[error("Internal IO error")]
    InternalIOError(#[from] std::io::Error),
}

pub async fn check_file_privilege(
    id: ID,
    key: &str,
    db_conn: &DatabaseConnection,
) -> Result<bool, DownloadError> {
    let id: u64 = id.into();
    let file = Files::find_by_id(key)
        .filter(files::Column::UserId.eq(id))
        .one(db_conn)
        .await?;
    Ok(file.is_some())
}

async fn download_impl(
    id: ID,
    req: pb::download::DownloadRequest,
    tx: &mpsc::Sender<Result<pb::download::DownloadResponse, Status>>,
    db_conn: &DbPool,
) -> Result<(), DownloadError> {
    let path = format!("{}/{}", "files_storage", &req.key);
    // check if the file can be downloaded by the user
    // TODO:check whether it belongs to a session which hold this file
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
        tx.send(Ok(pb::download::DownloadResponse { data: buf.to_vec() }))
            .await
            .ok();
    }
    Ok(())
}

pub async fn download(
    server: &RpcServer<impl EmailSender>,
    request: Request<pb::download::DownloadRequest>,
) -> Result<Response<DownloadStream>, tonic::Status> {
    let id = get_id_from_req(&request).unwrap();
    let req = request.into_inner();
    let (tx, rx) = mpsc::channel(16);
    let db_conn = server.db.clone();
    tokio::spawn(async move {
        match download_impl(id, req, &tx, &db_conn).await {
            Ok(_) => {}
            Err(e) => match e {
                DownloadError::PermissionDenied => {
                    tx.send(Err(tonic::Status::permission_denied("Permission Denied")))
                        .await
                        .ok();
                }
                _ => {
                    tracing::error!("{}", e);
                    tx.send(Err(tonic::Status::internal("Server Error")))
                        .await
                        .ok();
                }
            },
        }
    });
    let output_stream = ReceiverStream::new(rx);
    Ok(Response::new(Box::pin(output_stream) as DownloadStream))
}
