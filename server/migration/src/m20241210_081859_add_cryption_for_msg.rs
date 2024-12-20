use sea_orm_migration::{prelude::*, schema::*};

use crate::m20220101_000001_create_table::UserChatMsg;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(UserChatMsg::Table)
                    .add_column(boolean(UserChatMsg::IsEncrypted))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(UserChatMsg::Table)
                    .drop_column(UserChatMsg::IsEncrypted)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
