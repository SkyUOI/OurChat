use sea_orm_migration::{prelude::*, schema::*};

use crate::enums::{Permission, Role, RolePermissions, Session, User, UserRoleRelation};

#[derive(DeriveMigrationName)]
pub struct Migration;

const FK_NAME: &str = "FK_role_id";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create Role table
        manager
            .create_table(
                Table::create()
                    .table(Role::Table)
                    .if_not_exists()
                    .col(big_integer(Role::Id).primary_key().auto_increment())
                    .col(big_unsigned_null(Role::CreatorId))
                    .col(string(Role::Name))
                    .col(string_null(Role::Description))
                    .col(big_unsigned_null(Role::SessionId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Role::Table, Role::CreatorId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Role::Table, Role::SessionId)
                            .to(Session::Table, Session::SessionId)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create Permission table
        manager
            .create_table(
                Table::create()
                    .table(Permission::Table)
                    .if_not_exists()
                    .col(big_integer(Permission::Id).primary_key().auto_increment())
                    .col(string(Permission::Description).not_null())
                    .to_owned(),
            )
            .await?;

        // Create RolePermissions table
        manager
            .create_table(
                Table::create()
                    .table(RolePermissions::Table)
                    .if_not_exists()
                    .col(big_integer(RolePermissions::RoleId))
                    .col(big_integer(RolePermissions::PermissionId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(RolePermissions::Table, RolePermissions::RoleId)
                            .to(Role::Table, Role::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(RolePermissions::Table, RolePermissions::PermissionId)
                            .to(Permission::Table, Permission::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .primary_key(
                        Index::create()
                            .col(RolePermissions::RoleId)
                            .col(RolePermissions::PermissionId),
                    )
                    .to_owned(),
            )
            .await?;

        // Create UserRoleRelation table
        manager
            .create_table(
                Table::create()
                    .table(UserRoleRelation::Table)
                    .if_not_exists()
                    .col(big_unsigned(UserRoleRelation::SessionId))
                    .col(big_unsigned(UserRoleRelation::UserId))
                    .col(big_unsigned(UserRoleRelation::RoleId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserRoleRelation::Table, UserRoleRelation::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserRoleRelation::Table, UserRoleRelation::RoleId)
                            .to(Role::Table, Role::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserRoleRelation::Table, UserRoleRelation::SessionId)
                            .to(Session::Table, Session::SessionId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .primary_key(
                        Index::create()
                            .col(UserRoleRelation::SessionId)
                            .col(UserRoleRelation::UserId),
                    )
                    .to_owned(),
            )
            .await?;

        // Add foreign key from Session.DefaultRole to Role.Id
        manager
            .alter_table(
                Table::alter()
                    .table(Session::Table)
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

        // Insert predefined roles and permissions
        init_role_table(manager).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RolePermissions::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UserRoleRelation::Table).to_owned())
            .await?;
        // Drop foreign key from Session.DefaultRole to Role.Id
        manager
            .alter_table(
                Table::alter()
                    .table(Session::Table)
                    .drop_foreign_key(FK_NAME)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(Role::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Permission::Table).to_owned())
            .await?;
        Ok(())
    }
}

async fn init_role_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let conn = manager.get_connection();

    conn.execute_unprepared(
        r#"
INSERT INTO role (name, description) VALUES ('member', 'common member'), ('admin', 'have almost all permissions to manage the session'), ('owner', 'have all permissions to manage the session');
    "#,
    )
    .await?;
    conn.execute_unprepared(r#"
INSERT INTO permission (id, description) VALUES (1, 'send msg'), (2, 'recall other msg'), (3, 'ban user'),
(4, 'unban user'), (5, 'kick user'), (6, 'set title'), (7, 'set avatar'), (8, 'set description'), (9, 'delete session'), (10, 'set role'), (11, 'mute user'), (12, 'unmute user'), (13, 'accept join request'), (14, 'e2eeize and dee2eeize session');
    "#).await?;
    conn.execute_unprepared(r#"
    INSERT INTO role_permissions (role_id, permission_id) VALUES (3, 1), (3, 2), (3, 3), (3, 4), (3, 5), (3, 6), (3, 7), (3, 8), (3, 9), (3, 10), (3, 11), (3, 12), (3, 13), (3, 14),
(2, 1), (2, 2), (2, 3), (2, 4), (2, 5), (2, 11), (2, 12), (2, 13), (2, 14),
(1, 1), (1, 2);
"#).await?;
    Ok(())
}
