use crate::ShutdownRev;
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
    name: String,
    key: String,
    hash: String,
}

impl Record {
    pub fn new(name: impl Into<String>, key: impl Into<String>, hash: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            key: key.into(),
            hash: hash.into(),
        }
    }
}

struct TempUrl {
    hash: String,
    key: String,
}

struct UploadManager {
    records: DashMap<String, TempUrl>,
}

impl UploadManager {
    fn new() -> Self {
        Self {
            records: DashMap::new(),
        }
    }

    async fn add_record(
        data: web::Data<UploadManager>,
        mut request_receiver: mpsc::Receiver<Record>,
    ) -> anyhow::Result<()> {
        while let Some(record) = request_receiver.recv().await {
            data.records.insert(
                record.name,
                TempUrl {
                    hash: record.hash,
                    key: record.key,
                },
            );
        }
        Ok(())
    }

    fn get_records(&self, name: &str) -> Option<dashmap::mapref::one::Ref<'_, String, TempUrl>> {
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
    let data = body.freeze();
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let result = hasher.finalize();
    let hash = format!("{:x}", result);
    if hash != record.hash {
        return HttpResponse::BadRequest();
    }
    HttpResponse::Ok()
}

#[get("/download/{url}")]
async fn download(url: web::Path<String>, key: Query<File>) -> impl Responder {
    HttpResponse::Ok().content_type("file").body("body")
}

pub struct HttpServer {
    db: DatabaseConnection,
}

impl HttpServer {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn start(
        &mut self,
        ip: &str,
        http_port: u16,
        mut shutdown_receiver: ShutdownRev,
    ) -> anyhow::Result<(JoinHandle<anyhow::Result<()>>, mpsc::Sender<Record>)> {
        let shared_state = Data::new(UploadManager::new());
        let data_clone = shared_state.clone();
        let http_server = actix_web::HttpServer::new(move || {
            App::new().app_data(data_clone.clone()).service(upload)
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
