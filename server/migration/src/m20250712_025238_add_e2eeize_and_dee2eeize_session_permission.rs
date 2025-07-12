use sea_orm_migration::prelude::*;

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
    conn.execute_unprepared(
        r#"
DELETE FROM role_permissions WHERE permission_id = 14;
DELETE FROM permission WHERE id = 14;
"#,
    )
    .await?;
    Ok(())
}

async fn modify_up_role_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let conn = manager.get_connection();

    conn.execute_unprepared(
        r#"
INSERT INTO permission (id, description) VALUES (14, 'add e2ee or recall it on a session');
    "#,
    )
    .await?;
    conn.execute_unprepared(
        r#"
    INSERT INTO role_permissions (role_id, permission_id) VALUES (3, 14);
"#,
    )
    .await?;
    Ok(())
}
