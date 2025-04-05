use crate::m20220101_000001_create_table::User;
use crate::m20240812_083747_server_manager::{
    Authority, ServerManager, create_authority_table, create_server_manager_table,
};
use crate::m20250218_093632_server_manage_permission::ServerManagementRole;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Authority::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ServerManager::Table).to_owned())
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(ManagerRoleRelation::Table)
                    .col(big_unsigned(ManagerRoleRelation::UserId).primary_key())
                    .col(big_integer(ManagerRoleRelation::RoleId))
                    .foreign_key(
                        ForeignKey::create()
                            .to(User::Table, User::Id)
                            .from(ManagerRoleRelation::Table, ManagerRoleRelation::UserId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ManagerRoleRelation::Table, ManagerRoleRelation::RoleId)
                            .to(ServerManagementRole::Table, ServerManagementRole::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_authority_table(manager).await?;
        create_server_manager_table(manager).await?;
        manager
            .drop_table(Table::drop().table(ManagerRoleRelation::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum ManagerRoleRelation {
    Table,
    UserId,
    RoleId,
}
