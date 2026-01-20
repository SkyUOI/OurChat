//! Manage the file storage with simple ownership model

use entities::{files, prelude::*};
use parking_lot::RwLock;
use sea_orm::{ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::{fs::exists, sync::Arc};
use tokio::fs::remove_file;
use tokio_cron_scheduler::Job;
use tracing::{instrument, trace};

use crate::config::Cfg;

#[derive(Debug)]
pub struct FileSys {
    db_conn: DatabaseConnection,
    shared_cfg: Arc<RwLock<Cfg>>,
}

impl FileSys {
    pub fn new(db_conn: DatabaseConnection, shared_cfg: Arc<RwLock<Cfg>>) -> Self {
        Self {
            db_conn,
            shared_cfg,
        }
    }

    fn init() {
        if exists("files_storage").is_err() {
            std::fs::create_dir("files_storage").unwrap();
        }
    }

    pub async fn generate_job(&self) -> anyhow::Result<Job> {
        let db_conn = self.db_conn.clone();
        Self::init();
        let read = self.shared_cfg.read();
        let shared_cfg = self.shared_cfg.clone();
        trace!("Clean Cron: {}", read.main_cfg.auto_clean_duration);
        Ok(Job::new_async(
            read.main_cfg.auto_clean_duration.clone(),
            move |_uuid, _l| {
                let db_conn = db_conn.clone();
                let shared_cfg = shared_cfg.clone();
                Box::pin(async move {
                    auto_clean_files(shared_cfg, db_conn).await;
                })
            },
        )?)
    }
}

pub async fn clean_files(
    shared_cfg: Arc<RwLock<Cfg>>,
    db_conn: &mut DatabaseConnection,
) -> Result<(), FileStorageError> {
    // Query the file first
    let del_time = chrono::Utc::now() - shared_cfg.read().main_cfg.files_save_time;
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
pub async fn auto_clean_files(shared_cfg: Arc<RwLock<Cfg>>, mut connection: DatabaseConnection) {
    match clean_files(shared_cfg, &mut connection).await {
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

/// Delete a file immediately - no reference counting needed
#[instrument(skip(db_conn))]
pub async fn delete_file(
    key: impl Into<String> + std::fmt::Debug,
    db_conn: &impl ConnectionTrait,
) -> Result<(), FileStorageError> {
    let key = key.into();

    // Get the file to find its path
    let file = match Files::find_by_id(&key).one(db_conn).await? {
        Some(f) => f,
        None => return Err(FileStorageError::NotFound),
    };

    // Delete file from database
    Files::delete_by_id(&key).exec(db_conn).await?;

    // Delete file from filesystem
    remove_file(&file.path).await?;

    Ok(())
}

/// Generate hierarchical storage path for better filesystem performance
pub fn generate_hierarchical_path(
    base_path: &std::path::Path,
    user_id: u64,
    key: &str,
) -> std::path::PathBuf {
    // Use first 2 characters of user_id and key for directory structure
    let user_prefix = format!("{:02x}", user_id % 256);
    let key_prefix = &key[..2];

    base_path.join(&user_prefix).join(key_prefix).join(key)
}

/// Clean up orphaned files (files that exist on disk but not in database)
#[instrument(skip(db_conn))]
pub async fn cleanup_orphaned_files(
    db_conn: &impl ConnectionTrait,
    files_storage_path: &std::path::Path,
) -> Result<usize, FileStorageError> {
    let db_files = Files::find().all(db_conn).await?;
    let db_paths: std::collections::HashSet<String> =
        db_files.iter().map(|f| f.path.clone()).collect();

    let mut orphaned_count = 0;

    // Recursively scan files_storage_path
    let mut entries = tokio::fs::read_dir(files_storage_path).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            let path_str = path.to_string_lossy().to_string();
            if !db_paths.contains(&path_str) {
                tracing::info!("Removing orphaned file: {}", path_str);
                if let Err(e) = remove_file(&path).await {
                    tracing::error!("Failed to remove orphaned file {}: {}", path_str, e);
                } else {
                    orphaned_count += 1;
                }
            }
        }
    }

    Ok(orphaned_count)
}
