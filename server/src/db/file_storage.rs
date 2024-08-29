//! 管理文件储存

use derive::db_compatibility;
use sea_orm::DatabaseConnection;

#[db_compatibility]
pub async fn auto_clean_files(connection: DatabaseConnection) {
    loop {}
}
