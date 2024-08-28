use crate::ShutdownRev;
use actix_web::{
    post,
    web::{self, Data, Query},
    App, HttpResponse, HttpServer, Responder,
};
use dashmap::DashSet;
use serde::{Deserialize, Serialize};
use tokio::{select, sync::mpsc, task::JoinHandle};

#[derive(Debug, Serialize, Deserialize)]
struct File {
    key: String,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Record {
    name: String,
    key: String,
}

struct UploadManager {
    records: DashSet<Record>,
}

impl UploadManager {
    fn new() -> Self {
        Self {
            records: DashSet::new(),
        }
    }

    async fn add_record(
        data: web::Data<UploadManager>,
        mut request_receiver: mpsc::Receiver<Record>,
    ) -> anyhow::Result<()> {
        while let Some(record) = request_receiver.recv().await {
            data.records.insert(record);
        }
        Ok(())
    }
}

#[post("/upload/{name}")]
async fn upload(path: web::Path<String>, key: Query<File>) -> impl Responder {
    HttpResponse::Ok()
}

pub async fn start(
    ip: &str,
    http_port: u16,
    mut shutdown_receiver: ShutdownRev,
) -> anyhow::Result<(JoinHandle<anyhow::Result<()>>, mpsc::Sender<Record>)> {
    let shared_state = Data::new(UploadManager::new());
    let data_clone = shared_state.clone();
    let http_server = HttpServer::new(move || App::new().app_data(data_clone.clone()))
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
