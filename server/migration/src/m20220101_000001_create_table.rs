use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

pub const OCID_MAX_LEN: usize = 50;
pub const USERNAME_MAX_LEN: usize = 60;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
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
                    .col(text(User::Passwd))
                    .col(string_len(User::Name, USERNAME_MAX_LEN as u32))
                    .col(string_uniq(User::Email))
                    .col(timestamp_with_time_zone(User::Time))
                    .col(big_unsigned(User::ResourceUsed))
                    .col(integer(User::FriendLimit))
                    .col(integer(User::FriendsNum))
                    .col(string_null(User::Avatar))
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
        manager
            .create_table(
                Table::create()
                    .table(Friend::Table)
                    .if_not_exists()
                    .col(big_unsigned(Friend::UserId))
                    .col(big_unsigned(Friend::FriendId))
                    .col(string_len_null(Friend::DisplayName, 200))
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
                    .primary_key(Index::create().col(Friend::UserId).col(Friend::FriendId))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Session::Table)
                    .if_not_exists()
                    .col(big_unsigned(Session::SessionId))
                    .col(string_len(Session::Name, 200))
                    .col(integer(Session::Size))
                    .primary_key(Index::create().col(Session::SessionId))
                    .to_owned(),
            )
            .await?;
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
        manager
            .create_table(
                Table::create()
                    .table(UserChatMsg::Table)
                    .if_not_exists()
                    .col(
                        big_integer(UserChatMsg::ChatMsgId)
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(json_binary(UserChatMsg::MsgData))
                    .col(big_unsigned(UserChatMsg::SenderId))
                    .col(big_unsigned_null(UserChatMsg::SessionId))
                    .col(
                        timestamp_with_time_zone(UserChatMsg::Time)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserChatMsg::Table, UserChatMsg::SenderId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserChatMsg::Table, UserChatMsg::SessionId)
                            .to(Session::Table, Session::SessionId)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SessionRelation::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Friend::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UserChatMsg::Table).to_owned())
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

#[derive(DeriveIden)]
pub enum User {
    Table,
    Id,
    Ocid,
    Passwd,
    Name,
    Email,
    Time,
    ResourceUsed,
    FriendLimit,
    FriendsNum,
    PublicUpdateTime,
    UpdateTime,
    Avatar,
    Status,
}

#[derive(DeriveIden)]
pub enum Friend {
    Table,
    UserId,
    #[allow(clippy::enum_variant_names)]
    FriendId,
    DisplayName,
}

#[derive(DeriveIden)]
pub enum SessionRelation {
    Table,
    SessionId,
    UserId,
    DisplayName,
}

#[derive(DeriveIden)]
pub enum Session {
    Table,
    #[allow(clippy::enum_variant_names)]
    SessionId,
    Name,
    Size,
    AvatarKey,
    CreatedTime,
    UpdatedTime,
    Description,
    DefaultRole,
}

#[derive(DeriveIden)]
pub enum UserChatMsg {
    Table,
    ChatMsgId,
    MsgData,
    SenderId,
    SessionId,
    Time,
    IsEncrypted,
}
