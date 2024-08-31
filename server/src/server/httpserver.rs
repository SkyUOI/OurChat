use crate::{consts::ID, db::file_storage, ShutdownRev};
use actix_web::{
    get, post,
    web::{self, Data, Query},
    App, HttpRequest, HttpResponse, Responder,
};
use dashmap::DashMap;
use futures_util::StreamExt;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::{select, sync::mpsc, task::JoinHandle};

#[derive(Debug, Serialize, Deserialize)]
struct File {
    key: String,
}

#[derive(Debug)]
pub struct Record {
    url_name: String,
    key: String,
    hash: String,
    auto_clean: bool,
    user_id: ID,
}

impl Record {
    pub fn new(
        name: impl Into<String>,
        key: impl Into<String>,
        hash: impl Into<String>,
        auto_clean: bool,
        user_id: ID,
    ) -> Self {
        Self {
            url_name: name.into(),
            key: key.into(),
            hash: hash.into(),
            auto_clean,
            user_id,
        }
    }
}

struct UploadManager {
    records: DashMap<String, Record>,
}

impl UploadManager {
    fn new() -> Self {
        Self {
            records: DashMap::new(),
        }
    }

    async fn add_record(
        data: Data<UploadManager>,
        mut request_receiver: mpsc::Receiver<Record>,
    ) -> anyhow::Result<()> {
        while let Some(record) = request_receiver.recv().await {
            data.records.insert(record.url_name.clone(), record);
        }
        Ok(())
    }

    fn get_records(&self, name: &str) -> Option<dashmap::mapref::one::Ref<'_, String, Record>> {
        self.records.get(name)
    }
}

const KEY: &str = "Key";

#[post("/upload/{url}")]
async fn upload(
    req: HttpRequest,
    url: web::Path<String>,
    manager: web::Data<UploadManager>,
    mut payload: web::Payload,
    db_conn: web::Data<DatabaseConnection>,
) -> impl Responder {
    let url = url.into_inner();
    let key = match req.headers().get(KEY).and_then(|key| key.to_str().ok()) {
        None => {
            return HttpResponse::BadRequest();
        }
        Some(key) => key,
    };

    let mut body = bytes::BytesMut::new();
    // 获取临时url记录
    let record = match manager.get_records(&url) {
        None => {
            return HttpResponse::NotFound();
        }
        Some(data) => data,
    };
    // 验证key
    if record.key != key {
        return HttpResponse::Unauthorized();
    }
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

#[get("/download/{url}")]
async fn download(url: web::Path<String>, key: Query<File>) -> impl Responder {
    HttpResponse::Ok().content_type("file").body("body")
}

#[get("/status")]
async fn status() -> impl Responder {
    HttpResponse::Ok()
}

pub struct HttpServer {}

impl HttpServer {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn start(
        &mut self,
        ip: &str,
        http_port: u16,
        db_conn: DatabaseConnection,
        mut shutdown_receiver: ShutdownRev,
    ) -> anyhow::Result<(JoinHandle<anyhow::Result<()>>, mpsc::Sender<Record>)> {
        let shared_state = Data::new(UploadManager::new());
        let shared_db_conn = Data::new(db_conn);
        let data_clone = shared_state.clone();
        let http_server = actix_web::HttpServer::new(move || {
            App::new()
                .app_data(data_clone.clone())
                .app_data(shared_db_conn.clone())
                .service(upload)
        })
        .bind((ip, http_port))?
        .run();
        let http_server_handle = tokio::spawn(async move {
            select! {
                ret = http_server => {
                    tracing::info!("Http server exited internally");
                    ret?;
                }
                _ = shutdown_receiver.recv() => {
                    tracing::info!("Http server exited by shutdown signal");
                }
            }
            anyhow::Ok(())
        });
        let (request_sender, request_receiver) = mpsc::channel(100);
        tokio::spawn(UploadManager::add_record(shared_state, request_receiver));
        Ok((http_server_handle, request_sender))
    }

    pub async fn db_loop() -> anyhow::Result<()> {
        Ok(())
    }
}
