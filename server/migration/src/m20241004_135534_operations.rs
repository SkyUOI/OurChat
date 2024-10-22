use crate::m20220101_000001_create_table::User;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Operations::Table)
                    .if_not_exists()
                    .col(pk_auto(Operations::OperId))
                    .col(big_unsigned(Operations::Id))
                    .col(string(Operations::Operation))
                    .col(boolean(Operations::Once))
                    .col(timestamp_with_time_zone(Operations::ExpiresAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Operations::Table, Operations::Id)
                            .to(User::Table, User::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Operations::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Operations {
    Table,
    OperId,
    Id,
    Operation,
    Once,
    ExpiresAt,
}
