pub use sea_orm_migration::prelude::*;

pub mod m20220101_000001_create_table;
mod m20240812_083747_server_manager;
pub mod m20240829_010832_files;
mod m20241004_135534_operations;
mod m20241210_081859_add_cryption_for_msg;
mod m20241210_083126_create_index;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20240812_083747_server_manager::Migration),
            Box::new(m20240829_010832_files::Migration),
            Box::new(m20241004_135534_operations::Migration),
            Box::new(m20241210_081859_add_cryption_for_msg::Migration),
            Box::new(m20241210_083126_create_index::Migration),
        ]
    }
}
