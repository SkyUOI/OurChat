use sea_orm_migration::{prelude::*, schema::*};

use crate::m20220101_000001_create_table::{Friend, Session};

#[derive(DeriveMigrationName)]
pub struct Migration;

const FK_NAME: &str = "friend_session_id";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Friend::Table)
                    .add_column(big_unsigned(Friend::SessionId))
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name(FK_NAME)
                            .from_tbl(Friend::Table)
                            .from_col(Friend::SessionId)
                            .to_tbl(Session::Table)
                            .to_col(Session::SessionId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Friend::Table)
                    .drop_column(Friend::SessionId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
