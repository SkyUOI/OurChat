//! Manage the file storage with simple ownership model

use crate::SharedData;
use entities::{files, prelude::*};
use sea_orm::{ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::{fs::exists, sync::Arc};
use tokio::fs::remove_file;
use tokio::sync::RwLock;
use tokio_cron_scheduler::Job;
use tracing::{instrument, trace};

/// In-memory file cache for frequently accessed files
#[derive(Debug, Default)]
pub struct FileCache {
    cache: RwLock<HashMap<String, Vec<u8>>>,
    access_count: RwLock<HashMap<String, AtomicU32>>,
}

impl FileCache {
    /// Get file from cache if available
    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        let cache = self.cache.read().await;
        if let Some(data) = cache.get(key) {
            // Increment access count
            let access_map = self.access_count.read().await;
            if let Some(count) = access_map.get(key) {
                count.fetch_add(1, Ordering::Relaxed);
            }
            Some(data.clone())
        } else {
            None
        }
    }

    /// Add file to cache
    pub async fn put(&self, key: String, data: Vec<u8>) {
        let mut cache = self.cache.write().await;
        cache.insert(key.clone(), data);

        let mut access_map = self.access_count.write().await;
        access_map.insert(key, AtomicU32::new(1));
    }

    /// Remove file from cache
    pub async fn remove(&self, key: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(key);

        let mut access_map = self.access_count.write().await;
        access_map.remove(key);
    }

    /// Get access statistics
    pub async fn get_stats(&self) -> HashMap<String, u32> {
        let access_map = self.access_count.read().await;
        access_map
            .iter()
            .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed)))
            .collect()
    }
}

#[derive(Debug)]
pub struct FileSys {
    db_conn: Option<DatabaseConnection>,
    shared_data: Arc<SharedData>,
    file_cache: Arc<FileCache>,
}

impl FileSys {
    pub fn new(db_conn: DatabaseConnection, shared_data: Arc<SharedData>) -> Self {
        Self {
            db_conn: Some(db_conn),
            shared_data,
            file_cache: Arc::new(FileCache::default()),
        }
    }

    /// Get the file cache instance
    pub fn get_cache(&self) -> Arc<FileCache> {
        self.file_cache.clone()
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
