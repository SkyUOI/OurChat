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
                    .add_column(string_null(Session::AvatarKey))
                    .add_column(timestamp_with_time_zone(Session::UpdatedTime))
                    .add_column(timestamp_with_time_zone(Session::CreatedTime))
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
                    .drop_column(Session::AvatarKey)
                    .drop_column(Session::UpdatedTime)
                    .drop_column(Session::CreatedTime)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
