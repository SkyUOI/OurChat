use sea_orm_migration::{prelude::*, schema::*};

use crate::m20220101_000001_create_table::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Files::Table)
                    .if_not_exists()
                    .col(string(Files::Key))
                    .col(big_unsigned(Files::Date))
                    .col(boolean(Files::AutoClean))
                    .col(string(Files::Path))
                    .col(big_unsigned(Files::UserId))
                    .col(unsigned(Files::RefCnt).default(1))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Files::Table, Files::UserId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .primary_key(Index::create().col(Files::Key))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Files::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Files {
    Table,
    Key,
    Date,
    AutoClean,
    Path,
    UserId,
    RefCnt,
}
