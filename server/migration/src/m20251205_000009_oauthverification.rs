use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // OAuth fields (GithubId, OauthProvider, EmailVerified) are already added
        // in the core_tables migration (m20251205_000001_core_tables.rs)
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // Nothing to revert - fields are part of core tables
        Ok(())
    }
}
