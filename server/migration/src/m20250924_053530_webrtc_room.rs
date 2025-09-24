use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
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
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RTCRoom::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum RTCRoom {
    Table,
    RoomId,
    Title,
    UsersNum,
}
