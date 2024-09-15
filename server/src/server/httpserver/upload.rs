use crate::{
    db::file_storage,
    server::httpserver::{UploadManager, KEY},
};
use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use futures_util::StreamExt;
use sea_orm::DatabaseConnection;
use sha2::{Digest, Sha256};

#[post("/upload")]
pub async fn upload(
    req: HttpRequest,
    manager: web::Data<UploadManager>,
    mut payload: web::Payload,
    db_conn: web::Data<DatabaseConnection>,
) -> impl Responder {
    let key = match req.headers().get(KEY).and_then(|key| key.to_str().ok()) {
        None => {
            return HttpResponse::BadRequest();
        }
        Some(key) => key,
    };

    let mut body = bytes::BytesMut::new();
    // 获取临时url记录
    let record = match manager.get_records(key) {
        None => {
            return HttpResponse::NotFound();
        }
        Some(data) => data,
    };
    // 读取文件
    while let Some(chunk) = payload.next().await {
        let chunk = match chunk {
            Ok(data) => data,
            Err(_) => {
                return HttpResponse::InternalServerError();
            }
        };
        body.extend_from_slice(&chunk);
    }
    // 计算hash，并验证文件是否符合要求
    let mut data = body.freeze();
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let result = hasher.finalize();
    let hash = format!("{:x}", result);
    if hash != record.hash {
        return HttpResponse::BadRequest();
    }
    match file_storage::add_file(
        key,
        record.auto_clean,
        &mut data,
        record.user_id,
        &db_conn.into_inner(),
    )
    .await
    {
        Ok(_) => HttpResponse::Ok(),
        Err(_) => HttpResponse::InternalServerError(),
    };
    HttpResponse::Ok()
}
