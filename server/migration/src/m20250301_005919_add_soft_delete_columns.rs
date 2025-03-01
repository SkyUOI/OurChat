use sea_orm_migration::{
    prelude::*,
    schema::{integer, timestamp_with_time_zone_null},
};

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
                    .add_column_if_not_exists(
                        integer(User::AccountStatus).default(AccountStatus::Normal as i32),
                    )
                    .add_column_if_not_exists(timestamp_with_time_zone_null(User::DeletedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .drop_column(User::AccountStatus)
                    .drop_column(User::DeletedAt)
                    .to_owned(),
            )
            .await
    }
}

#[derive(num_enum::TryFromPrimitive, num_enum::IntoPrimitive)]
#[repr(i32)]
pub enum AccountStatus {
    Normal = 0,
    Deleted = 1,
}
