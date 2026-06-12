use sea_orm_migration::{prelude::*, schema::*};

use crate::enums::Files;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Files::Table)
                    .add_column_if_not_exists(
                        string_null(Files::ContentType), // MIME type
                    )
                    .add_column_if_not_exists(
                        string_null(Files::OriginalFilename), // original filename
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Files::Table)
                    .drop_column(Files::ContentType)
                    .drop_column(Files::OriginalFilename)
                    .to_owned(),
            )
            .await
    }
}
