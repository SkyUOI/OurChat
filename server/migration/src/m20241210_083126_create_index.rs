use sea_orm_migration::prelude::*;

use crate::m20220101_000001_create_table::UserChatMsg;

#[derive(DeriveMigrationName)]
pub struct Migration;

const USER_CHAR_MSG_TIME_IDX: &str = "idx_user_chat_msg_time";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(USER_CHAR_MSG_TIME_IDX)
                    .table(UserChatMsg::Table)
                    .col(UserChatMsg::Time)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name(USER_CHAR_MSG_TIME_IDX).to_owned())
            .await?;
        Ok(())
    }
}
