use sea_orm_migration::{
    prelude::*,
    schema::{big_unsigned, string, text, timestamp_with_time_zone},
};

use crate::m20220101_000001_create_table::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(Announcement::Table)
                    .col(big_unsigned(Announcement::Id).primary_key())
                    .col(string(Announcement::Title))
                    .col(text(Announcement::Content))
                    .col(
                        timestamp_with_time_zone(Announcement::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(big_unsigned(Announcement::PublisherId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Announcement::Table, Announcement::PublisherId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Announcement::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Announcement {
    Table,
    Id,
    Title,
    Content,
    CreatedAt,
    PublisherId,
}
