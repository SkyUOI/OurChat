//! 管理文件储存

use crate::{ShutdownRev, consts::ID, shared_state};
use derive::db_compatibility;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::{fs::exists, time::Duration};
use tokio::{
    fs::{File, remove_file},
    io::AsyncWriteExt,
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

    pub fn start(&mut self, mut shutodnw_receiver: ShutdownRev) {
        let db_conn = self.db_conn.take().unwrap();
        let db_conn_clone = db_conn.clone();
        Self::init();
        tokio::spawn(async move {
            select! {
                _ = auto_clean_files(db_conn_clone) => {}
                _ = shutodnw_receiver.recv() => {}
            }
        });
    }
}

// TODO: 测试该部分
#[db_compatibility]
pub async fn clean_files(db_conn: &mut DatabaseConnection) -> anyhow::Result<()> {
    use entities::files;
    // 先查询文件
    let del_time =
        chrono::Utc::now() - chrono::Duration::days(shared_state::get_file_save_days() as i64);
    let cond = files::Column::Date.lt(del_time.timestamp());
    let files = files::Entity::find()
        .filter(cond.clone())
        .all(db_conn)
        .await?;
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

/// 自动清理
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

#[db_compatibility]
pub async fn add_file(
    key: &str,
    auto_clean: bool,
    content: &mut bytes::Bytes,
    user_id: ID,
    db_conn: &DatabaseConnection,
) -> Result<(), AddFileError> {
    use entities::files;
    let timestamp = chrono::Utc::now().timestamp();
    let path = format!("{}/{}", "files_storage", key);
    let file = files::ActiveModel {
        key: sea_orm::Set(key.to_string()),
        path: sea_orm::Set(path.to_string()),
        date: sea_orm::Set(timestamp.try_into().unwrap()),
        auto_clean: sea_orm::Set(auto_clean.into()),
        user_id: sea_orm::Set(user_id.into()),
    };
    file.insert(db_conn).await?;
    let mut f = File::create(&path).await?;
    f.write_all_buf(content).await?;
    Ok(())
}
