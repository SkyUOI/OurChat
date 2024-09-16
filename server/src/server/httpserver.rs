mod download;
mod status;
mod upload;
mod verify;

use crate::{consts::ID, HttpSender, ShutdownRev};
use actix_web::{
    web::{self, Data},
    App,
};
use dashmap::DashMap;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tokio::{select, sync::mpsc, task::JoinHandle};
pub use upload::FileRecord;
pub use verify::VerifyRecord;

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
    ) -> anyhow::Result<(JoinHandle<anyhow::Result<()>>, HttpSender)> {
        let shared_state = Data::new(upload::UploadManager::new());
        let shared_db_conn = Data::new(db_conn);
        let data_clone = shared_state.clone();
        let http_server = actix_web::HttpServer::new(move || {
            let v1 = web::scope("/v1")
                .service(upload::upload)
                .service(status::status)
                .service(download::download)
                .configure(verify::config);
            App::new()
                .wrap(actix_web::middleware::Logger::default())
                .app_data(data_clone.clone())
                .app_data(shared_db_conn.clone())
                .service(v1)
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
        let (file_sender, file_receiver) = mpsc::channel(100);
        tokio::spawn(upload::UploadManager::add_record(
            shared_state,
            file_receiver,
        ));
        let (verify_sender, verify_receiver) = mpsc::channel(100);
        tokio::spawn(verify::VerifyManager::add_record(
            verify::MANAGER.clone(),
            verify_receiver,
        ));
        Ok((
            http_server_handle,
            HttpSender {
                file_record: file_sender,
                verify_record: verify_sender,
            },
        ))
    }

    pub async fn db_loop() -> anyhow::Result<()> {
        Ok(())
    }
}
