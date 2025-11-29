use sea_orm_migration::{prelude::*, schema::*};

use crate::enums::{Session, SessionInvitation, User};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SessionInvitation::Table)
                    .if_not_exists()
                    .col(pk_auto(SessionInvitation::Id))
                    .col(string(SessionInvitation::LeaveMessage))
                    .col(big_unsigned(SessionInvitation::Invitee))
                    .col(big_unsigned(SessionInvitation::Inviter))
                    .col(big_unsigned(SessionInvitation::SessionId))
                    .col(timestamp_with_time_zone(SessionInvitation::ExpireAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(SessionInvitation::Table, SessionInvitation::Invitee)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(SessionInvitation::Table, SessionInvitation::Inviter)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(SessionInvitation::Table, SessionInvitation::SessionId)
                            .to(Session::Table, Session::SessionId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SessionInvitation::Table).to_owned())
            .await
    }
}
