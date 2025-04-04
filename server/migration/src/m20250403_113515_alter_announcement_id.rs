use sea_orm_migration::{prelude::*, schema::*};

use crate::m20250329_120341_add_announcement_table::Announcement;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Announcement::Table)
                    .drop_column(Announcement::Id)
                    .add_column(big_integer(Announcement::Id).auto_increment().primary_key())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Announcement::Table)
                    .drop_column(Announcement::Id)
                    .add_column(big_unsigned(Announcement::Id).not_null().primary_key())
                    .to_owned(),
            )
            .await
    }
}
