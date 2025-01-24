//! Manage the file storage

use crate::{ShutdownRev, shared_state};
use entities::{files, prelude::*};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, ModelTrait,
    QueryFilter,
};
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
                _ = shutdown_receiver.wait_shutting_down() => {}
            }
        });
    }
}

pub async fn clean_files(db_conn: &mut DatabaseConnection) -> Result<(), FileStorageError> {
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
pub enum FileStorageError {
    #[error("Database error")]
    Db(#[from] sea_orm::DbErr),
    #[error("Internal IO error")]
    InternalIO(#[from] std::io::Error),
    #[error("Not found")]
    NotFound,
}

/// Reduce the file ref count
/// Each file has a ref count, when the ref count is 0, the file will be deleted
pub async fn dec_file_refcnt(
    key: impl Into<String>,
    db_conn: &impl ConnectionTrait,
) -> Result<(), FileStorageError> {
    let key = key.into();
    let mut file = match Files::find_by_id(key).one(db_conn).await? {
        Some(f) => f,
        None => return Err(FileStorageError::NotFound),
    };
    if file.ref_cnt <= 0 {
        tracing::error!("invalid file record:{:?}", file);
        file.delete(db_conn).await?;
    } else {
        file.ref_cnt -= 1;
        if file.ref_cnt == 0 {
            remove_file(&file.path).await?;
            file.delete(db_conn).await?;
        } else {
            let file: files::ActiveModel = file.into();
            file.update(db_conn).await?;
        }
    }
    Ok(())
}
