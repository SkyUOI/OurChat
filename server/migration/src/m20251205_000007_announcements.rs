use sea_orm_migration::{prelude::*, schema::*};

use crate::enums::{Announcement, AnnouncementMsg, MessageRecords, User};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create Announcement table
        manager
            .create_table(
                Table::create()
                    .table(Announcement::Table)
                    .if_not_exists()
                    .col(big_integer(Announcement::Id).primary_key().auto_increment())
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
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create AnnouncementMsg table
        manager
            .create_table(
                Table::create()
                    .table(AnnouncementMsg::Table)
                    .if_not_exists()
                    .col(big_integer(AnnouncementMsg::AnnouncementId).primary_key())
                    .foreign_key(
                        ForeignKey::create()
                            .from(AnnouncementMsg::Table, AnnouncementMsg::AnnouncementId)
                            .to(Announcement::Table, Announcement::Id),
                    )
                    .col(big_integer(AnnouncementMsg::MsgId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(AnnouncementMsg::Table, AnnouncementMsg::MsgId)
                            .to(MessageRecords::Table, MessageRecords::MsgId),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AnnouncementMsg::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Announcement::Table).to_owned())
            .await?;
        Ok(())
    }
}
