use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(ServerManagementPermission::Table)
                    .col(
                        big_integer(ServerManagementPermission::Id)
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(string(ServerManagementPermission::Description))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(ServerManagementRole::Table)
                    .col(
                        big_integer(ServerManagementRole::Id)
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(string(ServerManagementRole::Description))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(ServerManagementRolePermissions::Table)
                    .col(big_integer(ServerManagementRolePermissions::RoleId))
                    .col(big_integer(ServerManagementRolePermissions::PermissionId))
                    .primary_key(
                        Index::create()
                            .col(ServerManagementRolePermissions::RoleId)
                            .col(ServerManagementRolePermissions::PermissionId),
                    )
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
                    .to_owned(),
            )
            .await?;
        init_role_table(manager).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
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

#[derive(num_enum::TryFromPrimitive, num_enum::IntoPrimitive)]
#[repr(i64)]
pub enum PredefinedServerManagementPermission {
    PublishAnnouncement = 1,
    BanUser = 2,
    MuteUser = 3,
}

#[derive(num_enum::TryFromPrimitive, num_enum::IntoPrimitive)]
#[repr(i64)]
pub enum PredefinedServerManagementRole {
    Admin = 1,
}

#[derive(DeriveIden)]
enum ServerManagementPermission {
    Table,
    Id,
    Description,
}

#[derive(DeriveIden)]
pub enum ServerManagementRole {
    Table,
    Id,
    Description,
}

#[derive(DeriveIden)]
enum ServerManagementRolePermissions {
    Table,
    RoleId,
    PermissionId,
}

async fn init_role_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let conn = manager.get_connection();

    conn.execute_unprepared(
        r#"
INSERT INTO server_management_role (description) VALUES ('admin');
    "#,
    )
    .await?;
    conn.execute_unprepared(r#"
INSERT INTO server_management_permission (id, description) VALUES (1, 'publish announcement'), (2, 'ban user'), (3, 'mute user');
    "#).await?;
    conn.execute_unprepared(
        r#"
    INSERT INTO server_management_role_permissions (role_id, permission_id) VALUES (1, 1), (1, 2), (1, 3);
"#,
    )
    .await?;
    Ok(())
}
