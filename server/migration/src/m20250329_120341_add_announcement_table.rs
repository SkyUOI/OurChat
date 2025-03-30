use sea_orm_migration::{
    prelude::*,
    schema::{big_unsigned, string, text, timestamp},
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
                    .table(Announcement::Table)
                    .if_not_exists()
                    .col(big_unsigned(Announcement::Id).not_null().primary_key())
                    .col(string(Announcement::Title).not_null())
                    .col(text(Announcement::Content).not_null())
                    .col(
                        timestamp(Announcement::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(big_unsigned(Announcement::PublisherID).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-announcement-publisher")
                            .from(Announcement::Table, Announcement::PublisherID)
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
    PublisherID,
}
