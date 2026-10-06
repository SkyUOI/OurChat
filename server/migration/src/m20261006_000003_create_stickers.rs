use sea_orm_migration::{prelude::*, schema::*};

use crate::enums::{Files, Stickers, User};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Stickers::Table)
                    .if_not_exists()
                    .col(big_unsigned(Stickers::UserId))
                    .col(string(Stickers::FileKey))
                    .col(
                        timestamp_with_time_zone(Stickers::AddedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Stickers::Table, Stickers::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    // stickers disappear together with the underlying file
                    .foreign_key(
                        ForeignKey::create()
                            .from(Stickers::Table, Stickers::FileKey)
                            .to(Files::Table, Files::Key)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .primary_key(Index::create().col(Stickers::UserId).col(Stickers::FileKey))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Stickers::Table).to_owned())
            .await
    }
}
