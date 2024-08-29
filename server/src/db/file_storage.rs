//! 管理文件储存

use crate::{share_state, ShutdownRev};
use derive::db_compatibility;
use sea_orm::DatabaseConnection;
use std::time::Duration;
use tokio::{
    select,
    time::{sleep_until, Instant},
};

struct FileSys {
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

#[db_compatibility]
/// 自动清理
pub async fn auto_clean_files(connection: DatabaseConnection) {
    loop {
        let days = *share_state::AUTO_CLEAN_DURATION.lock();
        sleep_until(Instant::now() + Duration::from_days(days)).await;
    }
}
