//! Manage the file storage

use crate::{
    ShutdownRev,
    entities::{files, prelude::*},
    shared_state,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::{fs::exists, time::Duration};
use tokio::{
    fs::remove_file,
    select,
    time::{Instant, sleep_until},
};

pub struct FileSys {
    db_conn: Option<DatabaseConnection>,
}

impl FileSys {
    pub fn new(db_conn: DatabaseConnection) -> Self {
        Self {
            db_conn: Some(db_conn),
        }
    }

    fn init() {
        if exists("files_storage").is_err() {
            std::fs::create_dir("files_storage").unwrap();
        }
    }

    pub fn start(&mut self, mut shutdown_receiver: ShutdownRev) {
        let db_conn = self.db_conn.take().unwrap();
        let db_conn_clone = db_conn.clone();
        Self::init();
        tokio::spawn(async move {
            select! {
                _ = auto_clean_files(db_conn_clone) => {}
                _ = shutdown_receiver.wait_shutdowning() => {}
            }
        });
    }
}

pub async fn clean_files(db_conn: &mut DatabaseConnection) -> anyhow::Result<()> {
    // Query the file first
    let del_time =
        chrono::Utc::now() - chrono::Duration::days(shared_state::get_file_save_days() as i64);
    let cond = files::Column::Date.lt(del_time.timestamp());
    let files = Files::find().filter(cond.clone()).all(db_conn).await?;
    for i in files {
        match remove_file(i.path).await {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("delete file error: {}", e);
            }
        }
    }
    let res = files::Entity::delete_many()
        .filter(cond)
        .exec(db_conn)
        .await?;
    tracing::info!("delete {} files", res.rows_affected);
    Ok(())
}

/// auto clean files which is out-of-dated
#[tracing::instrument]
pub async fn auto_clean_files(mut connection: DatabaseConnection) {
    loop {
        let days = shared_state::get_auto_clean_duration();
        sleep_until(Instant::now() + Duration::from_days(days)).await;
        match clean_files(&mut connection).await {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("clean files error:{e}");
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AddFileError {
    #[error("Database error")]
    DbError(#[from] sea_orm::DbErr),
    #[error("Internal IO error")]
    InternalIOError(#[from] std::io::Error),
}
