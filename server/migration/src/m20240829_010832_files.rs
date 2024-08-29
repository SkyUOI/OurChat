use sea_orm_migration::{prelude::*, schema::*};

use crate::basic;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        basic::create_table(
            manager,
            Table::create()
                .table(Files::Table)
                .if_not_exists()
                .col(string(Files::Key))
                .col(big_unsigned(Files::Date))
                .col(boolean(Files::AutoClean))
                .col(string(Files::Path))
                .primary_key(Index::create().col(Files::Key)),
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Files::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Files {
    Table,
    Key,
    Date,
    AutoClean,
    Path,
}
