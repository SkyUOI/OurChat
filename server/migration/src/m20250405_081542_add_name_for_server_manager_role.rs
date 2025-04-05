use sea_orm_migration::{prelude::*, schema::*};

use crate::m20250218_093632_server_manage_permission::{
    PredefinedServerManagementPermission, PredefinedServerManagementRole,
    ServerManagementPermission, ServerManagementRole,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ServerManagementRole::Table)
                    .add_column(string_null(ServerManagementRole::Name))
                    .modify_column(string_null(ServerManagementRole::Description))
                    .to_owned(),
            )
            .await?;
        migrate_role_name(manager).await?;
        manager
            .alter_table(
                Table::alter()
                    .table(ServerManagementRole::Table)
                    .modify_column(string(ServerManagementRole::Name))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ServerManagementPermission::Table)
                    .add_column(string_null(ServerManagementPermission::Name))
                    .modify_column(string_null(ServerManagementPermission::Description))
                    .to_owned(),
            )
            .await?;
        migrate_permission_name(manager).await?;
        manager
            .alter_table(
                Table::alter()
                    .table(ServerManagementPermission::Table)
                    .modify_column(string(ServerManagementPermission::Name))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ServerManagementRole::Table)
                    .drop_column(ServerManagementRole::Name)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(ServerManagementPermission::Table)
                    .drop_column(ServerManagementPermission::Name)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

async fn migrate_role_name(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .get_connection()
        .execute_unprepared(&format!(
            r#"UPDATE server_management_role SET name = 'admin', description = 'have all permissions to manage the all things of server' WHERE id = {};"#,
            PredefinedServerManagementRole::Admin as i64,
        ))
        .await?;
    Ok(())
}

async fn migrate_permission_name(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .get_connection()
        .execute_unprepared(&format!(
            r#"UPDATE server_management_permission SET name = 'publish announcement', description = 'publish announcement to all user' WHERE id = {};
            UPDATE server_management_permission SET name = 'ban user', description = 'ban user from the server' WHERE id = {};
            UPDATE server_management_permission SET name = 'mute user', description = 'mute user from the server' WHERE id = {};
            "#,
            PredefinedServerManagementPermission::PublishAnnouncement as i64,
            PredefinedServerManagementPermission::BanUser as i64,
            PredefinedServerManagementPermission::MuteUser as i64,
        ))
        .await?;
    Ok(())
}
