use sea_orm_migration::{prelude::*, schema::*};

use crate::m20240829_010832_files::Files;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Files::Table)
                    .drop_column(Files::RefCnt)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Files::Table)
                    .add_column_if_not_exists(unsigned(Files::RefCnt))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
