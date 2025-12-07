use sea_orm_migration::{prelude::*, schema::*};

use crate::enums::{
    ManagerRoleRelation, ServerManagementPermission, ServerManagementRole,
    ServerManagementRolePermissions, User,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create ServerManagementPermission table
        manager
            .create_table(
                Table::create()
                    .table(ServerManagementPermission::Table)
                    .if_not_exists()
                    .col(
                        big_integer(ServerManagementPermission::Id)
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(string(ServerManagementPermission::Name))
                    .col(string_null(ServerManagementPermission::Description))
                    .to_owned(),
            )
            .await?;

        // Create ServerManagementRole table
        manager
            .create_table(
                Table::create()
                    .table(ServerManagementRole::Table)
                    .if_not_exists()
                    .col(
                        big_integer(ServerManagementRole::Id)
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(string(ServerManagementRole::Name))
                    .col(string_null(ServerManagementRole::Description))
                    .to_owned(),
            )
            .await?;

        // Create ServerManagementRolePermissions table
        manager
            .create_table(
                Table::create()
                    .table(ServerManagementRolePermissions::Table)
                    .if_not_exists()
                    .col(big_integer(ServerManagementRolePermissions::RoleId))
                    .col(big_integer(ServerManagementRolePermissions::PermissionId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                ServerManagementRolePermissions::Table,
                                ServerManagementRolePermissions::RoleId,
                            )
                            .to(ServerManagementRole::Table, ServerManagementRole::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                ServerManagementRolePermissions::Table,
                                ServerManagementRolePermissions::PermissionId,
                            )
                            .to(
                                ServerManagementPermission::Table,
                                ServerManagementPermission::Id,
                            )
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .primary_key(
                        Index::create()
                            .col(ServerManagementRolePermissions::RoleId)
                            .col(ServerManagementRolePermissions::PermissionId),
                    )
                    .to_owned(),
            )
            .await?;

        // Create ManagerRoleRelation table
        manager
            .create_table(
                Table::create()
                    .table(ManagerRoleRelation::Table)
                    .if_not_exists()
                    .col(big_unsigned(ManagerRoleRelation::UserId).primary_key())
                    .col(big_integer(ManagerRoleRelation::RoleId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(ManagerRoleRelation::Table, ManagerRoleRelation::UserId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ManagerRoleRelation::Table, ManagerRoleRelation::RoleId)
                            .to(ServerManagementRole::Table, ServerManagementRole::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Insert predefined server management roles and permissions
        init_server_management_tables(manager).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ManagerRoleRelation::Table).to_owned())
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(ServerManagementRolePermissions::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(ServerManagementPermission::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(ServerManagementRole::Table).to_owned())
            .await?;
        Ok(())
    }
}

async fn init_server_management_tables(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let conn = manager.get_connection();

    // Insert admin role
    conn.execute_unprepared(
        r#"
INSERT INTO server_management_role (name, description) VALUES ('admin', 'administrator');
    "#,
    )
    .await?;

    // Insert permissions
    conn.execute_unprepared(r#"
INSERT INTO server_management_permission (id, name, description) VALUES (1, 'publish_announcement', 'publish announcement'), (2, 'ban_user', 'ban user'), (3, 'mute_user', 'mute user');
    "#).await?;

    // Link role to permissions
    conn.execute_unprepared(r#"
INSERT INTO server_management_role_permissions (role_id, permission_id) VALUES (1, 1), (1, 2), (1, 3);
    "#).await?;

    Ok(())
}
