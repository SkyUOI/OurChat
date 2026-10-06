use sea_orm_migration::{prelude::*, schema::*};

use crate::enums::{RecallVoteRecords, RecallVotes, Session, User};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RecallVotes::Table)
                    .if_not_exists()
                    // BIGSERIAL primary key
                    .col(big_integer(RecallVotes::Id).auto_increment().primary_key())
                    // No foreign key on msg_id on purpose: recalled messages
                    // are deleted from message_records, but the vote rows
                    // must survive the recall they triggered.
                    .col(big_unsigned(RecallVotes::MsgId))
                    .col(big_unsigned(RecallVotes::SessionId))
                    .col(big_unsigned(RecallVotes::InitiatorId))
                    .col(unsigned(RecallVotes::YesCount).default(0))
                    .col(unsigned(RecallVotes::NoCount).default(0))
                    .col(unsigned(RecallVotes::EligibleCount).default(0))
                    .col(timestamp_with_time_zone(RecallVotes::Deadline))
                    .col(boolean(RecallVotes::Settled).default(false))
                    .col(boolean(RecallVotes::Passed).default(false))
                    .col(
                        timestamp_with_time_zone(RecallVotes::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(RecallVotes::Table, RecallVotes::SessionId)
                            .to(Session::Table, Session::SessionId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(RecallVotes::Table, RecallVotes::InitiatorId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Speeds up the "unsettled vote for a message" lookup in StartRecallVote
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_recall_votes_msg_id")
                    .table(RecallVotes::Table)
                    .col(RecallVotes::MsgId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(RecallVoteRecords::Table)
                    .if_not_exists()
                    .col(big_unsigned(RecallVoteRecords::VoteId))
                    .col(big_unsigned(RecallVoteRecords::UserId))
                    .col(boolean(RecallVoteRecords::Approve))
                    .col(
                        timestamp_with_time_zone(RecallVoteRecords::VotedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(RecallVoteRecords::Table, RecallVoteRecords::VoteId)
                            .to(RecallVotes::Table, RecallVotes::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(RecallVoteRecords::Table, RecallVoteRecords::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    // one vote per user per recall vote, enforced by the
                    // composite primary key
                    .primary_key(
                        Index::create()
                            .col(RecallVoteRecords::VoteId)
                            .col(RecallVoteRecords::UserId),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RecallVoteRecords::Table).to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name("idx_recall_votes_msg_id").to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(RecallVotes::Table).to_owned())
            .await
    }
}
