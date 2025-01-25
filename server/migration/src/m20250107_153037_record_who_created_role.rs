use crate::m20220101_000001_create_table::User;
use crate::m20241229_022701_add_role_for_session::Role;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

const FK_ID: &str = "FK_ID";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Role::Table)
                    .add_column(big_unsigned_null(Role::CreatorId))
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name(FK_ID)
                            .from_tbl(Role::Table)
                            .from_col(Role::CreatorId)
                            .to_tbl(User::Table)
                            .to_col(User::Id)
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
                    .table(Role::Table)
                    .drop_column(Role::CreatorId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
