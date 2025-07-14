use sea_orm_migration::{prelude::*, schema::*};

use crate::m20250404_114543_rename_user_chat_msg_to_message_records::MessageRecords;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(MessageRecords::Table)
                    .modify_column(big_unsigned_null(MessageRecords::SenderId))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(MessageRecords::Table)
                    .modify_column(big_unsigned(MessageRecords::SenderId))
                    .to_owned(),
            )
            .await
    }
}
