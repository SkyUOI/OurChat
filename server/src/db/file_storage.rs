//! 管理文件储存

use crate::{share_state, ShutdownRev};
use derive::db_compatibility;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::time::Duration;
use tokio::{
    fs::remove_file,
    select,
    time::{sleep_until, Instant},
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

    pub fn start(&mut self, mut shutodnw_receiver: ShutdownRev) {
        let db_conn = self.db_conn.take().unwrap();
        let db_conn_clone = db_conn.clone();
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
        chrono::Utc::now() - chrono::Duration::days(share_state::get_file_save_days() as i64);
    let cond = files::Column::Date.lt(del_time.timestamp());
    let files = files::Entity::find()
        .filter(cond.clone())
        .all(db_conn)
        .await?;
    for i in files {
        match remove_file(i.path).await {
            Ok(_) => {}
            Err(e) => {
                log::error!("delete file error: {}", e);
            }
        }
    }
    let res = files::Entity::delete_many()
        .filter(cond)
        .exec(db_conn)
        .await?;
    log::info!("delete {} files", res.rows_affected);
    Ok(())
}

/// 自动清理
#[tracing::instrument]
pub async fn auto_clean_files(mut connection: DatabaseConnection) {
    loop {
        let days = share_state::get_auto_clean_duration();
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
}

#[db_compatibility]
pub async fn add_file(
    key: &str,
    path: &str,
    auto_clean: bool,
    db_conn: &mut DatabaseConnection,
) -> Result<(), AddFileError> {
    use entities::files;
    let timestamp = chrono::Utc::now().timestamp();
    let file = files::ActiveModel {
        key: sea_orm::Set(key.to_string()),
        path: sea_orm::Set(path.to_string()),
        date: sea_orm::Set(timestamp.try_into().unwrap()),
        auto_clean: sea_orm::Set(auto_clean.into()),
    };
    file.insert(db_conn).await?;
    Ok(())
}
