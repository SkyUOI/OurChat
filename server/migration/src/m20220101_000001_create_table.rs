use sea_orm_migration::{prelude::*, schema::*};

use crate::basic;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        basic::create_table(
            manager,
            Table::create()
                .table(User::Table)
                .if_not_exists()
                .col(big_unsigned(User::Id))
                .col(char_len_uniq(User::Ocid, 10))
                .col(text(User::Passwd))
                .col(char_len(User::Name, 200))
                .col(char_len_uniq(User::Email, 120))
                .col(big_unsigned(User::Time))
                .col(integer(User::ResourceUsed))
                .col(integer(User::FriendLimit))
                .col(integer(User::FriendsNum))
                .primary_key(Index::create().col(User::Id)),
        )
        .await?;
        basic::create_table(
            manager,
            Table::create()
                .table(Friend::Table)
                .if_not_exists()
                .col(big_unsigned(Friend::UserId))
                .col(big_unsigned(Friend::FriendId))
                .col(char_len(Friend::Name, 200))
                .foreign_key(
                    ForeignKey::create()
                        .from(Friend::Table, Friend::FriendId)
                        .to(User::Table, User::Id),
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(Friend::Table, Friend::UserId)
                        .to(User::Table, User::Id),
                )
                .primary_key(Index::create().col(Friend::UserId)),
        )
        .await?;
        basic::create_table(
            manager,
            Table::create()
                .table(Session::Table)
                .if_not_exists()
                .col(big_unsigned(Session::SessionId))
                .col(char_len(Session::GroupName, 200))
                .primary_key(Index::create().col(Session::SessionId)),
        )
        .await?;
        basic::create_table(
            manager,
            Table::create()
                .table(SessionRelation::Table)
                .if_not_exists()
                .col(big_unsigned(SessionRelation::SessionId))
                .col(big_unsigned(SessionRelation::UserId))
                .col(char_len(SessionRelation::NickName, 200))
                .col(char_len(SessionRelation::GroupName, 200))
                .foreign_key(
                    ForeignKey::create()
                        .from(SessionRelation::Table, SessionRelation::UserId)
                        .to(User::Table, User::Id),
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(SessionRelation::Table, SessionRelation::SessionId)
                        .to(Session::Table, Session::SessionId),
                )
                .primary_key(Index::create().col(SessionRelation::SessionId)),
        )
        .await?;
        basic::create_table(
            manager,
            Table::create()
                .table(UserChatMsg::Table)
                .if_not_exists()
                .col(big_unsigned(UserChatMsg::ChatMsgId))
                .col(unsigned(UserChatMsg::MsgType))
                .col(string_len(UserChatMsg::MsgData, 8000))
                .col(big_unsigned(UserChatMsg::SenderId))
                .foreign_key(
                    ForeignKey::create()
                        .from(UserChatMsg::Table, UserChatMsg::SenderId)
                        .to(User::Table, User::Id),
                )
                .primary_key(Index::create().col(UserChatMsg::ChatMsgId)),
        )
        .await?;
        basic::create_table(
            manager,
            Table::create()
                .table(UserChatMsgRelation::Table)
                .if_not_exists()
                .col(big_unsigned(UserChatMsgRelation::UserId))
                .col(big_unsigned(UserChatMsgRelation::ChatMsgId))
                .foreign_key(
                    ForeignKey::create()
                        .from(UserChatMsgRelation::Table, UserChatMsgRelation::UserId)
                        .to(User::Table, User::Id),
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(UserChatMsgRelation::Table, UserChatMsgRelation::ChatMsgId)
                        .to(UserChatMsg::Table, UserChatMsg::ChatMsgId),
                )
                .primary_key(Index::create().col(UserChatMsgRelation::ChatMsgId)),
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Friend::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(SessionRelation::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Session::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UserChatMsgRelation::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UserChatMsg::Table).to_owned())
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
}

#[derive(DeriveIden)]
enum Friend {
    Table,
    UserId,
    #[allow(clippy::enum_variant_names)]
    FriendId,
    Name,
}

#[derive(DeriveIden)]
enum SessionRelation {
    Table,
    SessionId,
    UserId,
    NickName,
    GroupName,
}

#[derive(DeriveIden)]
enum Session {
    Table,
    #[allow(clippy::enum_variant_names)]
    SessionId,
    GroupName,
}

#[derive(DeriveIden)]
enum UserChatMsgRelation {
    Table,
    UserId,
    ChatMsgId,
}

#[derive(DeriveIden)]
enum UserChatMsg {
    Table,
    ChatMsgId,
    MsgType,
    MsgData,
    SenderId,
}
