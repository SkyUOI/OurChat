use crate::m20220101_000001_create_table::{Session, User};
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Role::Table)
                    .if_not_exists()
                    .col(big_unsigned(Role::Id).primary_key())
                    .col(string(Role::Description).not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Permission::Table)
                    .if_not_exists()
                    .col(big_unsigned(Permission::Id).primary_key())
                    .col(string(Permission::Description).not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(RolePermissions::Table)
                    .if_not_exists()
                    .col(big_unsigned(RolePermissions::RoleId))
                    .col(big_unsigned(RolePermissions::PermissionId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(RolePermissions::Table, RolePermissions::RoleId)
                            .to(Role::Table, Role::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(RolePermissions::Table, RolePermissions::PermissionId)
                            .to(Permission::Table, Permission::Id),
                    )
                    .primary_key(
                        Index::create()
                            .col(RolePermissions::RoleId)
                            .col(RolePermissions::PermissionId),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(UserRoleRelation::Table)
                    .if_not_exists()
                    .col(big_unsigned(UserRoleRelation::UserId))
                    .col(big_unsigned(UserRoleRelation::SessionId))
                    .col(big_unsigned(UserRoleRelation::RoleId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserRoleRelation::Table, UserRoleRelation::UserId)
                            .to(User::Table, User::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserRoleRelation::Table, UserRoleRelation::RoleId)
                            .to(Role::Table, Role::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserRoleRelation::Table, UserRoleRelation::SessionId)
                            .to(Session::Table, Session::SessionId),
                    )
                    .primary_key(
                        Index::create()
                            .col(UserRoleRelation::UserId)
                            .col(UserRoleRelation::SessionId),
                    )
                    .to_owned(),
            )
            .await?;
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
        manager
            .drop_table(Table::drop().table(Role::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Permission::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum UserRoleRelation {
    Table,
    UserId,
    SessionId,
    RoleId,
}

#[derive(DeriveIden)]
enum Role {
    Table,
    Id,
    Description,
}

#[derive(DeriveIden)]
enum Permission {
    Table,
    Id,
    Description,
}

#[derive(DeriveIden)]
enum RolePermissions {
    Table,
    RoleId,
    PermissionId,
}

#[derive(num_enum::TryFromPrimitive, num_enum::IntoPrimitive)]
#[repr(u64)]
pub enum PreDefinedPermissions {
    SendMsg = 1,
    RecallMsg = 2,
    BanUser = 3,
    UnbanUser = 4,
    KickUser = 5,
    SetTitle = 6,
    SetAvatar = 7,
    SetDescription = 8,
    DeleteSession = 9,
    SetRole = 10,
}

#[derive(num_enum::TryFromPrimitive, num_enum::IntoPrimitive)]
#[repr(u64)]
pub enum PreDefinedRoles {
    Member = 1,
    Admin = 2,
    Owner = 3,
}

async fn init_role_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let conn = manager.get_connection();

    conn.execute_unprepared(&format!(
        r#"
INSERT INTO role (id, description) VALUES ({}, 'member'), ({}, 'admin'), ({}, 'owner');
    "#,
        PreDefinedRoles::Member as u32,
        PreDefinedRoles::Admin as u32,
        PreDefinedRoles::Owner as u32
    ))
    .await?;
    conn.execute_unprepared(r#"
INSERT INTO permission (id, description) VALUES (1, 'send msg'), (2, 'recall other msg'), (3, 'ban user'),
(4, 'unban user'), (5, 'kick user'), (6, 'set title'), (7, 'set avatar'), (8, 'set description'), (9, 'delete session'), (10, 'set role');
    "#).await?;
    conn.execute_unprepared(r#"
    INSERT INTO role_permissions (role_id, permission_id) VALUES (3, 1), (3, 2), (3, 3), (3, 4), (3, 5), (3, 6), (3, 7), (3, 8), (3, 9), (3, 10),
(2, 1), (2, 2), (2, 3), (2, 4), (2, 5),
(1, 1);
"#).await?;
    Ok(())
}
