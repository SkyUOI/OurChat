mod status;
pub mod verify;

use std::sync::Arc;

use crate::{DbPool, HttpSender, ShutdownSdr, component::EmailSender};
use actix_web::{
    App,
    web::{self},
};
use tokio::{select, task::JoinHandle};

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
        shutdown_sender: ShutdownSdr,
    ) -> anyhow::Result<(JoinHandle<anyhow::Result<()>>, HttpSender)> {
        let shared_state_moved = shared_data.clone();
        let db_conn_clone = db_conn.clone();
        let http_server = actix_web::HttpServer::new(move || {
            let v1 = web::scope("/v1")
                .service(status::status)
                .configure(verify::config);
            App::new()
                .wrap(actix_web::middleware::Logger::default())
                .app_data(db_conn_clone.clone())
                .app_data(shared_state_moved.clone())
                .service(v1)
        })
        .listen(listener)?
        .run();
        let mut shutdown_receiver = shutdown_sender.new_receiver("http server", "http server");
        let http_server_handle = tokio::spawn(async move {
            select! {
                ret = http_server => {
                    tracing::info!("Http server exited internally");
                    ret?;
                }
                _ = shutdown_receiver.wait_shutdowning() => {
                    tracing::info!("Http server exited by shutdown signal");
                }
            }
            anyhow::Ok(())
        });
        let mut shutdown_receiver = shutdown_sender.new_receiver("notifier cleaner", "");
        tokio::spawn(async move {
            select! {
                _ = verify::regularly_clean_notifier(
                    shared_data.clone(),
                    db_conn.redis_pool.clone(),
                ) => {},
                _ = shutdown_receiver.wait_shutdowning() => {},
            }
        });
        Ok((http_server_handle, HttpSender {}))
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
