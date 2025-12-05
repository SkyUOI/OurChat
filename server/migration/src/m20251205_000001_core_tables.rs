use sea_orm_migration::{prelude::*, schema::*};

use crate::constants::{OCID_MAX_LEN, USERNAME_MAX_LEN};
use crate::enums::{Friend, MessageRecords, Session, SessionRelation, User};
use crate::predefined;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create User table
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(big_unsigned(User::Id))
                    .col(string_len_uniq(
                        User::Ocid,
                        OCID_MAX_LEN.try_into().unwrap(),
                    ))
                    .col(text_null(User::Passwd)) // nullable for OAuth
                    .col(string_len(User::Name, USERNAME_MAX_LEN as u32))
                    .col(string_uniq(User::Email))
                    .col(timestamp_with_time_zone(User::Time))
                    .col(big_unsigned(User::ResourceUsed))
                    .col(integer(User::FriendLimit))
                    .col(integer(User::FriendsNum))
                    .col(string_null(User::Avatar))
                    .col(
                        integer(User::AccountStatus)
                            .default(predefined::AccountStatus::Active as i32),
                    )
                    .col(timestamp_with_time_zone_null(User::DeletedAt))
                    .col(binary(User::PublicKey))
                    .col(string_null(User::GithubId).unique_key())
                    .col(string_null(User::OauthProvider))
                    .col(boolean(User::EmailVerified))
                    .col(
                        timestamp_with_time_zone(User::PublicUpdateTime)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(User::UpdateTime)
                            .default(Expr::current_timestamp()),
                    )
                    .primary_key(Index::create().col(User::Id))
                    .to_owned(),
            )
            .await?;

        // Create Session table
        manager
            .create_table(
                Table::create()
                    .table(Session::Table)
                    .if_not_exists()
                    .col(big_unsigned(Session::SessionId))
                    .col(string_len(Session::Name, 200))
                    .col(integer(Session::Size))
                    .col(string_null(Session::AvatarKey))
                    .col(
                        timestamp_with_time_zone(Session::CreatedTime)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Session::UpdatedTime)
                            .default(Expr::current_timestamp()),
                    )
                    .col(string_null(Session::Description))
                    .col(
                        big_unsigned(Session::DefaultRole)
                            .default(predefined::PredefinedRoles::Member as u64),
                    )
                    .col(boolean(Session::E2EEOn).default(false))
                    .col(
                        timestamp_with_time_zone(Session::RoomKeyTime)
                            .default(Expr::current_timestamp()),
                    )
                    .col(boolean(Session::LeavingToProcess))
                    .primary_key(Index::create().col(Session::SessionId))
                    .to_owned(),
            )
            .await?;

        // Create Friend table
        manager
            .create_table(
                Table::create()
                    .table(Friend::Table)
                    .if_not_exists()
                    .col(big_unsigned(Friend::UserId))
                    .col(big_unsigned(Friend::FriendId))
                    .col(big_unsigned(Friend::SessionId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Friend::Table, Friend::FriendId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Friend::Table, Friend::UserId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Friend::Table, Friend::SessionId)
                            .to(Session::Table, Session::SessionId)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .primary_key(Index::create().col(Friend::UserId).col(Friend::FriendId))
                    .to_owned(),
            )
            .await?;

        // Create SessionRelation table
        manager
            .create_table(
                Table::create()
                    .table(SessionRelation::Table)
                    .if_not_exists()
                    .col(big_unsigned(SessionRelation::SessionId))
                    .col(big_unsigned(SessionRelation::UserId))
                    .col(string_len(SessionRelation::DisplayName, 200).default(Expr::value("")))
                    .foreign_key(
                        ForeignKey::create()
                            .from(SessionRelation::Table, SessionRelation::UserId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(SessionRelation::Table, SessionRelation::SessionId)
                            .to(Session::Table, Session::SessionId)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .primary_key(
                        Index::create()
                            .col(SessionRelation::SessionId)
                            .col(SessionRelation::UserId),
                    )
                    .to_owned(),
            )
            .await?;

        // Create MessageRecords table (formerly UserChatMsg)
        manager
            .create_table(
                Table::create()
                    .table(MessageRecords::Table)
                    .if_not_exists()
                    .col(
                        big_integer(MessageRecords::MsgId)
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(json_binary(MessageRecords::MsgData))
                    .col(big_unsigned_null(MessageRecords::SenderId))
                    .col(big_unsigned_null(MessageRecords::SessionId))
                    .col(
                        timestamp_with_time_zone(MessageRecords::Time)
                            .default(Expr::current_timestamp()),
                    )
                    .col(boolean(MessageRecords::IsEncrypted).default(false))
                    .col(boolean(MessageRecords::IsAllUser).default(false))
                    .foreign_key(
                        ForeignKey::create()
                            .from(MessageRecords::Table, MessageRecords::SenderId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(MessageRecords::Table, MessageRecords::SessionId)
                            .to(Session::Table, Session::SessionId)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on MessageRecords.Time
        manager
            .create_index(
                Index::create()
                    .name("idx_message_records_time")
                    .table(MessageRecords::Table)
                    .col(MessageRecords::Time)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_message_records_time").to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(MessageRecords::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(SessionRelation::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Friend::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Session::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await?;
        Ok(())
    }
}
