use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // The `member` role (role_id = 1) must not hold the RecallMsg permission
        // (permission_id = 2, "recall other msg"): that permission is reserved for
        // privileged roles (admin/owner), which are not bound by the recall time limit.
        // Members can still recall their own messages within `recall_time_limit`.
        conn.execute_unprepared(
            r#"
DELETE FROM role_permissions
WHERE role_id = 1 AND permission_id = 2;
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared(
            r#"
INSERT INTO role_permissions (role_id, permission_id) VALUES (1, 2)
ON CONFLICT (role_id, permission_id) DO NOTHING;
            "#,
        )
        .await?;

        Ok(())
    }
}
