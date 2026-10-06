use base::constants::{ID, SessionID};
use entities::{recall_vote_records, recall_votes};
use sea_orm::sea_query::ExprTrait;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter,
    QuerySelect, sea_query::Expr,
};

/// Error type for recall vote database operations.
#[derive(Debug, thiserror::Error)]
pub enum VoteDbError {
    #[error("database error:{0:?}")]
    Db(#[from] DbErr),
    /// the user has already voted on this recall vote (unique key violation)
    #[error("duplicate vote record")]
    DuplicateVote,
}

/// A recall vote together with a marker telling whether this call performed
/// the state transition (used to decide who broadcasts).
#[derive(Debug, Clone)]
pub struct VoteUpdate {
    pub vote: recall_votes::Model,
    /// true if this call settled the vote (won the conditional update)
    pub settled_now: bool,
}

/// Get a recall vote by its id.
pub async fn get_vote_by_id<T: ConnectionTrait>(
    vote_id: u64,
    db_conn: &T,
) -> Result<Option<recall_votes::Model>, DbErr> {
    recall_votes::Entity::find_by_id(vote_id as i64)
        .one(db_conn)
        .await
}

/// Find the unsettled recall vote of a message, if any.
///
/// At most one unsettled vote is allowed per message; this is enforced by
/// `StartRecallVote` checking this query before inserting a new vote.
pub async fn get_open_vote_by_msg_id<T: ConnectionTrait>(
    msg_id: u64,
    db_conn: &T,
) -> Result<Option<recall_votes::Model>, DbErr> {
    recall_votes::Entity::find()
        .filter(recall_votes::Column::MsgId.eq(msg_id as i64))
        .filter(recall_votes::Column::Settled.eq(false))
        .limit(1)
        .one(db_conn)
        .await
}

/// Insert a new recall vote.
pub async fn insert_vote<T: ConnectionTrait>(
    msg_id: u64,
    session_id: SessionID,
    initiator_id: ID,
    eligible_count: u32,
    deadline: chrono::DateTime<chrono::FixedOffset>,
    db_conn: &T,
) -> Result<recall_votes::Model, DbErr> {
    let vote = recall_votes::ActiveModel {
        msg_id: ActiveValue::Set(msg_id as i64),
        session_id: ActiveValue::Set(session_id.into()),
        initiator_id: ActiveValue::Set(initiator_id.into()),
        eligible_count: ActiveValue::Set(eligible_count as i64),
        deadline: ActiveValue::Set(deadline),
        ..Default::default()
    };
    vote.insert(db_conn).await
}

/// Whether the given user has already voted on the given recall vote.
pub async fn has_voted<T: ConnectionTrait>(
    vote_id: u64,
    user_id: ID,
    db_conn: &T,
) -> Result<bool, DbErr> {
    Ok(
        recall_vote_records::Entity::find_by_id((vote_id as i64, user_id.into()))
            .one(db_conn)
            .await?
            .is_some(),
    )
}

/// Record a user's vote and atomically increment the matching counter of the
/// vote, but only while the vote is still unsettled.
///
/// Returns [`VoteDbError::DuplicateVote`] if the user has already voted. The
/// counter increment is guarded by `settled == false`, so a vote racing with
/// its settlement is recorded but not counted.
pub async fn cast_vote_record<T: ConnectionTrait>(
    vote: &recall_votes::Model,
    user_id: ID,
    approve: bool,
    db_conn: &T,
) -> Result<recall_votes::Model, VoteDbError> {
    let record = recall_vote_records::ActiveModel {
        vote_id: ActiveValue::Set(vote.id),
        user_id: ActiveValue::Set(user_id.into()),
        approve: ActiveValue::Set(approve),
        ..Default::default()
    };
    if let Err(e) = record.insert(db_conn).await {
        // the composite primary key makes every user vote at most once
        return match crate::db::helper::is_conflict(&e) {
            true => Err(VoteDbError::DuplicateVote),
            false => Err(e.into()),
        };
    }
    let counter = if approve {
        recall_votes::Column::YesCount
    } else {
        recall_votes::Column::NoCount
    };
    recall_votes::Entity::update_many()
        .col_expr(counter, Expr::col(counter).add(1))
        .filter(recall_votes::Column::Id.eq(vote.id))
        .filter(recall_votes::Column::Settled.eq(false))
        .exec(db_conn)
        .await?;
    Ok(get_vote_by_id(vote.id as u64, db_conn)
        .await?
        .expect("vote just updated"))
}

/// Settle a vote if it has not been settled yet.
///
/// The update is conditional on `settled == false`, so exactly one concurrent
/// caller wins; the returned [`VoteUpdate`] tells whether this call was the
/// winner (`settled_now == true`).
pub async fn settle_vote<T: ConnectionTrait>(
    vote_id: u64,
    passed: bool,
    db_conn: &T,
) -> Result<VoteUpdate, DbErr> {
    let result = recall_votes::Entity::update_many()
        .col_expr(recall_votes::Column::Settled, Expr::value(true))
        .col_expr(recall_votes::Column::Passed, Expr::value(passed))
        .filter(recall_votes::Column::Id.eq(vote_id as i64))
        .filter(recall_votes::Column::Settled.eq(false))
        .exec(db_conn)
        .await?;
    let settled_now = result.rows_affected == 1;
    let vote = get_vote_by_id(vote_id, db_conn)
        .await?
        .expect("vote just updated");
    Ok(VoteUpdate { vote, settled_now })
}
