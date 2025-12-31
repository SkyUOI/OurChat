use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // Insert new AssignRole server management permission
        conn.execute_unprepared(
            r#"
INSERT INTO server_management_permission (id, name, description) VALUES
(9, 'assign_role', 'assign server management roles to users')
ON CONFLICT (id) DO NOTHING;
        "#,
        )
        .await?;

        // Link permission to admin role (role_id = 1)
        conn.execute_unprepared(
            r#"
INSERT INTO server_management_role_permissions (role_id, permission_id) VALUES
(1, 9)
ON CONFLICT (role_id, permission_id) DO NOTHING;
        "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // Remove permission link from admin role
        conn.execute_unprepared(
            r#"
DELETE FROM server_management_role_permissions
WHERE permission_id = 9 AND role_id = 1;
        "#,
        )
        .await?;

        // Remove the permission
        conn.execute_unprepared(
            r#"
DELETE FROM server_management_permission WHERE id = 9;
        "#,
        )
        .await?;

        Ok(())
    }
}
