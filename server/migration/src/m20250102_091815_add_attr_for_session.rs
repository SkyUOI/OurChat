use sea_orm_migration::{prelude::*, schema::*};

use crate::{
    m20220101_000001_create_table::Session,
    m20241229_022701_add_role_for_session::{PreDefinedRoles, Role},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

const FK_NAME: &str = "FK_role_id";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Session::Table)
                    .add_column(string(Session::Description).default(""))
                    .add_column(big_unsigned(Session::DefaultRole).default(PreDefinedRoles::Member))
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name(FK_NAME)
                            .from_tbl(Session::Table)
                            .from_col(Session::DefaultRole)
                            .to_tbl(Role::Table)
                            .to_col(Role::Id)
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
                    .table(Session::Table)
                    .drop_column(Session::Description)
                    .drop_column(Session::DefaultRole)
                    .drop_foreign_key(Alias::new(FK_NAME))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
