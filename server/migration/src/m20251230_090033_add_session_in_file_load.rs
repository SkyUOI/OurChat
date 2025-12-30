use sea_orm_migration::{prelude::*, schema::*};

use crate::enums::{Files, Session};

#[derive(DeriveMigrationName)]
pub struct Migration;

pub static FOREIGN_KEY: &str = "fk_files_session_id";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Files::Table)
                    .add_column(big_unsigned_null(Files::SessionId))
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name(FOREIGN_KEY)
                            .to_tbl(Session::Table)
                            .to_col(Session::SessionId)
                            .from_tbl(Files::Table)
                            .from_col(Files::SessionId)
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
                    .table(Files::Table)
                    .drop_foreign_key(FOREIGN_KEY)
                    .drop_column(Files::SessionId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
