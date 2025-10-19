use sea_orm_migration::prelude::*;

use crate::m20241229_022701_add_role_for_session::{PredefinedPermissions, PredefinedRoles};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        modify_up_role_table(manager).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        modify_down_role_table(manager).await
    }
}

async fn modify_down_role_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let conn = manager.get_connection();
    let permission_id: u64 = PredefinedPermissions::E2eeizeAndDee2eeizeSession.into();
    conn.execute_unprepared(&format!(
        r#"
DELETE FROM role_permissions WHERE permission_id = {};
DELETE FROM permission WHERE id = {};
"#,
        permission_id, permission_id
    ))
    .await?;
    Ok(())
}

async fn modify_up_role_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let conn = manager.get_connection();
    let owner: u64 = PredefinedRoles::Owner.into();
    let permission_id: u64 = PredefinedPermissions::E2eeizeAndDee2eeizeSession.into();

    conn.execute_unprepared(&format!(
        r#"
INSERT INTO permission (id, description) VALUES ({}, 'add e2ee or recall it on a session');
    "#,
        permission_id
    ))
    .await?;
    conn.execute_unprepared(&format!(
        r#"
    INSERT INTO role_permissions (role_id, permission_id) VALUES ({}, {});
"#,
        owner, permission_id
    ))
    .await?;
    Ok(())
}
