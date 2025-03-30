use sea_orm_migration::prelude::*;

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
                    .col(
                        ColumnDef::new(Announcement::Id)
                            .big_unsigned()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Announcement::Title).string().not_null())
                    .col(ColumnDef::new(Announcement::Content).text().not_null())
                    .col(
                        ColumnDef::new(Announcement::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Announcement::PublisherID)
                            .big_unsigned()
                            .not_null(),
                    )
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
