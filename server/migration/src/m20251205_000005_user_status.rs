use sea_orm_migration::{prelude::*, schema::*};

use crate::enums::UserStatus;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserStatus::Table)
                    .if_not_exists()
                    .col(string(UserStatus::Name).primary_key())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserStatus::Table).to_owned())
            .await
    }
}
