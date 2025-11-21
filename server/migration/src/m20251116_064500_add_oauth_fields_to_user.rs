use sea_orm_migration::{prelude::*, schema::*};

use crate::m20220101_000001_create_table::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .add_column_if_not_exists(string_uniq(User::GithubId).default(""))
                    .add_column_if_not_exists(string_null(User::OauthProvider))
                    .modify_column(text_null(User::Passwd))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .drop_column(User::GithubId)
                    .drop_column(User::OauthProvider)
                    .modify_column(text(User::Passwd).not_null())
                    .to_owned(),
            )
            .await
    }
}
