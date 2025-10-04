//! Manage the file storage

use crate::SharedData;
use entities::{files, prelude::*};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, ModelTrait,
    QueryFilter,
};
use std::{fs::exists, sync::Arc};
use tokio::fs::remove_file;
use tokio_cron_scheduler::Job;
use tracing::trace;

pub struct FileSys {
    db_conn: Option<DatabaseConnection>,
    shared_data: Arc<SharedData>,
}

impl FileSys {
    pub fn new(db_conn: DatabaseConnection, shared_data: Arc<SharedData>) -> Self {
        Self {
            db_conn: Some(db_conn),
            shared_data,
        }
    }

    fn init() {
        if exists("files_storage").is_err() {
            std::fs::create_dir("files_storage").unwrap();
        }
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        let db_conn = self.db_conn.take().unwrap();
        let shared = self.shared_data.clone();
        Self::init();
        let cron = self
            .shared_data
            .cfg
            .main_cfg
            .auto_clean_duration
            .to_string();
        trace!("Clean Cron: {}", cron);
        // add seconds
        let cron = format!("0 {cron}");
        self.shared_data
            .sched
            .lock()
            .await
            .add(Job::new_async(cron, move |_uuid, _l| {
                let db_conn = db_conn.clone();
                let shared = shared.clone();
                Box::pin(async move {
                    auto_clean_files(&shared, db_conn).await;
                })
            })?)
            .await?;
        Ok(())
    }
}

pub async fn clean_files(
    shared: &Arc<SharedData>,
    db_conn: &mut DatabaseConnection,
) -> Result<(), FileStorageError> {
    // Query the file first
    let del_time = chrono::Utc::now() - shared.cfg.main_cfg.file_save_time;
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

/// auto clean files that are out-of-dated
#[tracing::instrument]
pub async fn auto_clean_files(shared: &Arc<SharedData>, mut connection: DatabaseConnection) {
    match clean_files(shared, &mut connection).await {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("clean files error:{e}");
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

/// Increase the file ref count
pub async fn inc_file_refcnt(
    key: impl Into<String>,
    db_conn: &impl ConnectionTrait,
) -> Result<(), FileStorageError> {
    let key = key.into();
    let mut file = match Files::find_by_id(key).one(db_conn).await? {
        Some(f) => f,
        None => return Err(FileStorageError::NotFound),
    };
    file.ref_cnt += 1;
    let file: files::ActiveModel = file.into();
    file.update(db_conn).await?;
    Ok(())
}
