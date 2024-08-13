use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ServerManager::Table)
                    .if_not_exists()
                    .col(string_uniq(ServerManager::Id))
                    .col(big_unsigned(ServerManager::IdToUser))
                    .col(string(ServerManager::Passwd))
                    .primary_key(Index::create().col(ServerManager::Id))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Authority::Table)
                    .if_not_exists()
                    .col(pk_auto(Authority::Id))
                    .col(string(Authority::Description))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ServerManager::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Authority::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum ServerManager {
    Table,
    Id,
    IdToUser,
    Passwd,
}

#[derive(DeriveIden)]
enum Authority {
    Table,
    Id,
    Description
}
