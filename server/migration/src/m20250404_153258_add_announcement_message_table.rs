use sea_orm_migration::{prelude::*, schema::*};

use crate::{
    m20250329_120341_add_announcement_table::Announcement,
    m20250404_114543_rename_user_chat_msg_to_message_records::MessageRecords,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AnnouncementMsg::Table)
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
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AnnouncementMsg::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AnnouncementMsg {
    Table,
    AnnouncementId,
    MsgId,
}
