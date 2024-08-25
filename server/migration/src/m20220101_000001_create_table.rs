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
                .col(char_len(User::Passwd, 64))
                .col(char_len(User::Name, 15))
                .col(char_len_uniq(User::Email, 120))
                .col(big_unsigned(User::Time))
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
                .col(char_len(Friend::Name, 15))
                .primary_key(Index::create().col(Friend::UserId)),
        )
        .await?;
        basic::create_table(
            manager,
            Table::create()
                .table(Chat::Table)
                .if_not_exists()
                .col(big_unsigned(Chat::GroupId))
                .col(big_unsigned(Chat::UserId))
                .col(char_len(Chat::Name, 15))
                .col(char_len(Chat::GroupName, 30))
                .primary_key(Index::create().col(Chat::GroupId)),
        )
        .await?;
        basic::create_table(
            manager,
            Table::create()
                .table(ChatGroup::Table)
                .if_not_exists()
                .col(big_unsigned(ChatGroup::GroupId))
                .col(char_len(ChatGroup::GroupName, 30))
                .primary_key(Index::create().col(ChatGroup::GroupId)),
        )
        .await?;
        basic::create_table(
            manager,
            Table::create()
                .table(UserChatMsg::Table)
                .if_not_exists()
                .col(big_unsigned(UserChatMsg::UserId))
                .col(unsigned(UserChatMsg::ChatMsgId))
                .primary_key(Index::create().col(UserChatMsg::ChatMsgId)),
        )
        .await?;
        basic::create_table(
            manager,
            Table::create()
                .table(UserChatId::Table)
                .if_not_exists()
                .col(big_unsigned(UserChatId::ChatMsgId))
                .col(unsigned(UserChatId::MsgType))
                .col(string_len(UserChatId::MsgData, 8000))
                .col(big_unsigned(UserChatId::SenderId))
                .primary_key(Index::create().col(UserChatId::ChatMsgId)),
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
            .drop_table(Table::drop().table(Chat::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ChatGroup::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UserChatMsg::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UserChatId::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Ocid,
    Passwd,
    Name,
    Email,
    Time,
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
enum Chat {
    Table,
    GroupId,
    UserId,
    Name,
    GroupName,
}

#[derive(DeriveIden)]
enum ChatGroup {
    Table,
    GroupId,
    GroupName,
}

#[derive(DeriveIden)]
enum UserChatMsg {
    Table,
    UserId,
    ChatMsgId,
}

#[derive(DeriveIden)]
enum UserChatId {
    Table,
    ChatMsgId,
    MsgType,
    MsgData,
    SenderId,
}
