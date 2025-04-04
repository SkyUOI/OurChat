use sea_orm_migration::prelude::*;

use crate::m20220101_000001_create_table::UserChatMsg;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .rename_table(
                Table::rename()
                    .table(UserChatMsg::Table, MessageRecords::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(MessageRecords::Table)
                    .rename_column(Alias::new("chat_msg_id"), Alias::new("msg_id"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // manager
        //     .alter_table(
        //         Table::alter()
        //             .table(MessageRecords::Table)
        //             .rename_column(Alias::new("msg_id"), Alias::new("chat_msg_id"))
        //             .to_owned(),
        //     )
        //     .await?;
        manager
            .rename_table(
                Table::rename()
                    .table(MessageRecords::Table, UserChatMsg::Table)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
#[derive(DeriveIden)]
pub enum MessageRecords {
    Table,
    IsAllUser,
    MsgId,
}
