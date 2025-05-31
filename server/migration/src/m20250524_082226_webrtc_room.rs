use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
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
        manager
            .create_table(
                Table::create()
                    .table(WebrtcRoomMember::Table)
                    .if_not_exists()
                    .col(uuid(WebrtcRoomMember::RoomId))
                    .col(big_unsigned(WebrtcRoomMember::UserId))
                    .col(string_null(WebrtcRoomMember::Name))
                    .primary_key(
                        Index::create()
                            .col(WebrtcRoomMember::RoomId)
                            .col(WebrtcRoomMember::UserId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(WebrtcRoomMember::Table, WebrtcRoomMember::RoomId)
                            .to(WebrtcRoom::Table, WebrtcRoom::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
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
            .drop_table(Table::drop().table(WebrtcRoom::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum WebrtcRoom {
    Table,
    Id,
    Title,
    Description,
}

#[derive(DeriveIden)]
enum WebrtcRoomMember {
    Table,
    RoomId,
    UserId,
    Name,
}
