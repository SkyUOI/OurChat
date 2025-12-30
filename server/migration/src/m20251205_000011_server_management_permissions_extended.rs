use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // Insert new server management permissions
        conn.execute_unprepared(
            r#"
INSERT INTO server_management_permission (id, name, description) VALUES
(4, 'view_configuration', 'view server configuration'),
(5, 'modify_configuration', 'modify server configuration'),
(6, 'view_monitoring', 'view server monitoring metrics'),
(7, 'view_users', 'view user list and details'),
(8, 'manage_sessions', 'manage sessions (view, delete, remove users)')
ON CONFLICT (id) DO NOTHING;
        "#,
        )
        .await?;

        // Link new permissions to admin role (role_id = 1)
        conn.execute_unprepared(
            r#"
INSERT INTO server_management_role_permissions (role_id, permission_id) VALUES
(1, 4), (1, 5), (1, 6), (1, 7), (1, 8)
ON CONFLICT (role_id, permission_id) DO NOTHING;
        "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // Remove permission links from admin role
        conn.execute_unprepared(
            r#"
DELETE FROM server_management_role_permissions
WHERE permission_id IN (4, 5, 6, 7, 8) AND role_id = 1;
        "#,
        )
        .await?;

        // Remove the new permissions
        conn.execute_unprepared(
            r#"
DELETE FROM server_management_permission WHERE id IN (4, 5, 6, 7, 8);
        "#,
        )
        .await?;

        Ok(())
    }
}
