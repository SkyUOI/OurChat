use sea_orm::DatabaseBackend;
use sea_orm_migration::prelude::*;

pub(crate) async fn create_table(
    manager: &SchemaManager<'_>,
    mut table: &mut TableCreateStatement,
) -> Result<(), DbErr> {
    if let DatabaseBackend::Sqlite = manager.get_database_backend() {
    } else if let DatabaseBackend::MySql = manager.get_database_backend() {
        table = table.character_set("utf8mb4");
    } else {
        panic!("not support");
    }
    manager.create_table(table.to_owned()).await?;
    Ok(())
}
