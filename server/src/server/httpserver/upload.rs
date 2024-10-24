use crate::{consts::ID, db::file_storage};
use actix_multipart::form::MultipartForm;
use actix_web::{
    HttpRequest, HttpResponse, Responder, post,
    web::{self, Data},
};
use dashmap::DashMap;
use futures_util::StreamExt;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256 as Sha256};
use tokio::{io::AsyncWriteExt, sync::mpsc};

use super::FileUploadForm;

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
    MultipartForm(payload): MultipartForm<FileUploadForm>,
    manager: Data<UploadManager>,
    db_conn: Data<DatabaseConnection>,
) -> impl Responder {
    let key = &payload.metadata.key;

    // get temporyory url record
    let record = match manager.get_records(key) {
        None => {
            return HttpResponse::NotFound();
        }
        Some(data) => data,
    };
    // create record and file
    let f = match file_storage::add_file(
        key,
        record.auto_clean,
        record.user_id,
        &db_conn.into_inner(),
    )
    .await
    {
        Ok(f) => f,
        Err(_) => return HttpResponse::InternalServerError(),
    };
    // read file
    // f.write_all(payload.)
    // calculate hash and verify whether files is correct
    // let mut data = body.freeze();
    // let mut hasher = Sha256::new();
    // hasher.update(&data);
    // let result = hasher.finalize();
    // let hash = format!("{:x}", result);
    // if hash != record.hash {
    //     return HttpResponse::BadRequest();
    // }
    //
    manager.remove_record(key);
    HttpResponse::Ok()
}
