mod download;
mod status;
mod upload;
mod verify;

use std::sync::Arc;

use crate::{DbPool, HttpSender, ShutdownRev, component::EmailSender};
use actix_web::{
    App,
    web::{self, Data},
};
use sea_orm::DatabaseConnection;
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
        listener: std::net::TcpListener,
        db_conn: DbPool,
        shared_data: Arc<crate::SharedData<impl EmailSender>>,
        mut shutdown_receiver: ShutdownRev,
    ) -> anyhow::Result<(JoinHandle<anyhow::Result<()>>, HttpSender)> {
        let upload_manager = Data::new(upload::UploadManager::new());
        let db_conn_clone = db_conn.clone();
        let data_clone = upload_manager.clone();
        let shared_state_moved = shared_data.clone();
        let http_server = actix_web::HttpServer::new(move || {
            let v1 = web::scope("/v1")
                .service(upload::upload)
                .service(status::status)
                .service(download::download)
                .configure(verify::config);
            App::new()
                .wrap(actix_web::middleware::Logger::default())
                .app_data(data_clone.clone())
                .app_data(db_conn.clone())
                .app_data(shared_state_moved.clone())
                .service(v1)
        })
        .listen(listener)?
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
            upload_manager,
            file_receiver,
        ));
        let (verify_sender, verify_receiver) = mpsc::channel(100);
        tokio::spawn(verify::add_record(
            db_conn_clone,
            shared_data,
            verify_receiver,
        ));
        Ok((http_server_handle, HttpSender {
            file_record: file_sender,
            verify_record: verify_sender,
        }))
    }

    pub async fn db_loop() -> anyhow::Result<()> {
        Ok(())
    }
}

impl Default for HttpServer {
    fn default() -> Self {
        Self::new()
    }
}
