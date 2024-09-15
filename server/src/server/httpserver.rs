mod download;
mod status;
mod upload;

use crate::{consts::ID, ShutdownRev};
use actix_web::{web::Data, App};
use dashmap::DashMap;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tokio::{select, sync::mpsc, task::JoinHandle};

#[derive(Debug, Serialize, Deserialize)]
struct File {
    key: String,
}

#[derive(Debug)]
pub struct Record {
    url_name: String,
    hash: String,
    auto_clean: bool,
    user_id: ID,
}

impl Record {
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
                .wrap(actix_web::middleware::Logger::default())
                .app_data(data_clone.clone())
                .app_data(shared_db_conn.clone())
                .service(upload::upload)
                .service(status::status)
                .service(download::download)
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
