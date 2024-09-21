use crate::{consts::ID, db::file_storage, server::httpserver::KEY};
use actix_web::{
    HttpRequest, HttpResponse, Responder, post,
    web::{self, Data},
};
use dashmap::DashMap;
use futures_util::StreamExt;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::mpsc;

#[derive(Debug, Serialize, Deserialize)]
struct File {
    key: String,
}

#[derive(Debug)]
pub struct FileRecord {
    url_name: String,
    hash: String,
    auto_clean: bool,
    user_id: ID,
}

impl FileRecord {
    pub fn new(
        name: impl Into<String>,
        hash: impl Into<String>,
        auto_clean: bool,
        user_id: ID,
    ) -> Self {
        Self {
            url_name: name.into(),
            hash: hash.into(),
            auto_clean,
            user_id,
        }
    }
}

pub struct UploadManager {
    // TODO:add timeout
    records: DashMap<String, FileRecord>,
}

impl UploadManager {
    pub fn new() -> Self {
        Self {
            records: DashMap::new(),
        }
    }

    pub async fn add_record(
        data: Data<UploadManager>,
        mut request_receiver: mpsc::Receiver<FileRecord>,
    ) -> anyhow::Result<()> {
        while let Some(record) = request_receiver.recv().await {
            data.records.insert(record.url_name.clone(), record);
        }
        Ok(())
    }

    fn get_records(&self, name: &str) -> Option<dashmap::mapref::one::Ref<'_, String, FileRecord>> {
        self.records.get(name)
    }

    fn remove_record(&self, name: &str) {
        self.records.remove(name);
    }
}

#[post("/upload")]
pub async fn upload(
    req: HttpRequest,
    manager: Data<UploadManager>,
    mut payload: web::Payload,
    db_conn: Data<DatabaseConnection>,
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
    manager.remove_record(key);
    HttpResponse::Ok()
}
