use sea_orm_migration::{prelude::*, schema::*};

use crate::{m20220101_000001_create_table::Session, m20241229_022701_add_role_for_session::Role};

#[derive(DeriveMigrationName)]
pub struct Migration;

pub static FK_NAME: &str = "role_session_key";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Role::Table)
                    .add_column(big_unsigned_null(Role::SessionId))
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name(FK_NAME)
                            .from_tbl(Role::Table)
                            .from_col(Role::SessionId)
                            .to_tbl(Session::Table)
                            .to_col(Session::SessionId),
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
                    .table(Role::Table)
                    .to_owned()
                    .drop_column(Role::SessionId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
