use sea_orm_migration::{prelude::*, schema::*};

use crate::enums::{RTCRoom, User, WebrtcRoom, WebrtcRoomMember};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create WebrtcRoom table
        manager
            .create_table(
                Table::create()
                    .table(WebrtcRoom::Table)
                    .if_not_exists()
                    .col(pk_uuid(WebrtcRoom::Id))
                    .col(string_null(WebrtcRoom::Title))
                    .col(string_null(WebrtcRoom::Description))
                    .to_owned(),
            )
            .await?;

        // Create RTCRoom table
        manager
            .create_table(
                Table::create()
                    .table(RTCRoom::Table)
                    .if_not_exists()
                    .col(big_unsigned(RTCRoom::RoomId))
                    .col(string(RTCRoom::Title))
                    .col(unsigned(RTCRoom::UsersNum))
                    .primary_key(Index::create().col(RTCRoom::RoomId))
                    .to_owned(),
            )
            .await?;

        // Create WebrtcRoomMember table
        manager
            .create_table(
                Table::create()
                    .table(WebrtcRoomMember::Table)
                    .if_not_exists()
                    .col(uuid(WebrtcRoomMember::RoomId))
                    .col(big_unsigned(WebrtcRoomMember::UserId))
                    .col(string_null(WebrtcRoomMember::Name))
                    .foreign_key(
                        ForeignKey::create()
                            .from(WebrtcRoomMember::Table, WebrtcRoomMember::RoomId)
                            .to(WebrtcRoom::Table, WebrtcRoom::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(WebrtcRoomMember::Table, WebrtcRoomMember::UserId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .primary_key(
                        Index::create()
                            .col(WebrtcRoomMember::RoomId)
                            .col(WebrtcRoomMember::UserId),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(WebrtcRoomMember::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(RTCRoom::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(WebrtcRoom::Table).to_owned())
            .await?;
        Ok(())
    }
}
