use sea_orm_migration::{prelude::*, schema::*};

use crate::m20220101_000001_create_table::Session;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Session::Table)
                    .add_column(string(Session::Description))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Session::Table)
                    .drop_column(Session::Description)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
