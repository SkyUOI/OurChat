use sea_orm_migration::{prelude::*, schema::*};

use crate::m20220101_000001_create_table::Session;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Session::Table)
                    .add_column(
                        timestamp_with_time_zone(Session::RoomKeyTime)
                            .default(Expr::current_timestamp()),
                    )
                    .add_column(boolean(Session::LeavingToProcess))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Session::Table)
                    .drop_column(Session::RoomKeyTime)
                    .drop_column(Session::LeavingToProcess)
                    .to_owned(),
            )
            .await
    }
}
