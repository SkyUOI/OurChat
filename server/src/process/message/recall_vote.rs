//! Recall votes (#33).
//!
//! Members without the `RecallMsg` permission may start a vote to recall a
//! message. A vote passes as soon as more than half of the eligible members
//! (all members except the sender of the message) approve it; it fails early
//! as soon as at least half of the eligible members reject it (it then can no
//! longer reach a majority), and fails at its deadline otherwise. Expiry is
//! settled lazily by whichever entry point (`VoteRecall`/`GetRecallVote`)
//! touches the vote first.

use crate::db::messages::{MsgError, get_msg_by_id};
use crate::db::recall_vote::{
    VoteDbError, cast_vote_record, get_open_vote_by_msg_id, get_vote_by_id, insert_vote,
    settle_vote,
};
use crate::db::session::{get_members, if_permission_exist, in_session};
use crate::process::error_msg::{
    CANNOT_RECALL_BY_VOTE, NOT_IN_SESSION, PERMISSION_DENIED, RECALL_PERMISSION_HELD, SERVER_ERROR,
    VOTE_EXPIRED, VOTE_SETTLED, exist, not_found,
};
use crate::process::{Dest, message_insert_and_transmit};
use crate::server::RpcServer;
use anyhow::Context;
use base::constants::{ID, SessionID};
use entities::recall_votes;
use migration::predefined::PredefinedPermissions;
use pb::service::ourchat::msg_delivery::recall_vote::v1::{
    GetRecallVoteRequest, GetRecallVoteResponse, RecallVoteNotification, StartRecallVoteRequest,
    StartRecallVoteResponse, VoteRecallRequest, VoteRecallResponse,
};
use pb::service::ourchat::msg_delivery::v1::fetch_msgs_response::RespondEventType;
use tonic::{Request, Response, Status};

pub async fn start_recall_vote(
    server: &RpcServer,
    id: ID,
    request: Request<StartRecallVoteRequest>,
) -> Result<Response<StartRecallVoteResponse>, Status> {
    match start_recall_vote_internal(server, id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => Err(map_err(e)),
    }
}

pub async fn vote_recall(
    server: &RpcServer,
    id: ID,
    request: Request<VoteRecallRequest>,
) -> Result<Response<VoteRecallResponse>, Status> {
    match vote_recall_internal(server, id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => Err(map_err(e)),
    }
}

pub async fn get_recall_vote(
    server: &RpcServer,
    id: ID,
    request: Request<GetRecallVoteRequest>,
) -> Result<Response<GetRecallVoteResponse>, Status> {
    match get_recall_vote_internal(server, id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => Err(map_err(e)),
    }
}

#[derive(Debug, thiserror::Error)]
enum VoteErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("unknown error:{0:?}")]
    Unknown(#[from] anyhow::Error),
    #[error("status:{0:?}")]
    Status(#[from] Status),
}

impl From<MsgError> for VoteErr {
    fn from(value: MsgError) -> Self {
        match value {
            MsgError::DbError(db_err) => Self::Db(db_err),
            MsgError::NotFound => Self::Status(Status::not_found(not_found::MSG)),
            MsgError::UnknownError(error) => Self::Unknown(error),
            MsgError::SerdeError(error) => Self::Unknown(error.into()),
            // unreachable through the message lookups used here
            MsgError::PermissionDenied | MsgError::TimeLimitExceeded => {
                Self::Status(Status::internal(SERVER_ERROR))
            }
        }
    }
}

impl From<VoteDbError> for VoteErr {
    fn from(value: VoteDbError) -> Self {
        match value {
            VoteDbError::Db(db_err) => Self::Db(db_err),
            VoteDbError::DuplicateVote => {
                Self::Status(Status::already_exists(exist::RECALL_VOTE_RECORD))
            }
        }
    }
}

impl From<crate::process::MsgInsTransmitErr> for VoteErr {
    fn from(value: crate::process::MsgInsTransmitErr) -> Self {
        match value {
            crate::process::MsgInsTransmitErr::Db(db_err) => Self::Db(db_err),
            crate::process::MsgInsTransmitErr::Unknown(error) => Self::Unknown(error),
            // unreachable: broadcasting a vote event requires no privilege
            crate::process::MsgInsTransmitErr::PermissionDenied
            | crate::process::MsgInsTransmitErr::NotFound => {
                Self::Status(Status::internal(SERVER_ERROR))
            }
        }
    }
}

fn map_err(e: VoteErr) -> Status {
    match e {
        VoteErr::Db(_) | VoteErr::Unknown(_) => {
            tracing::error!("{}", e);
            Status::internal(SERVER_ERROR)
        }
        VoteErr::Status(status) => status,
    }
}

/// Build the full vote state notification from a vote row.
fn vote_notification(vote: &recall_votes::Model) -> RecallVoteNotification {
    RecallVoteNotification {
        vote_id: vote.id as u64,
        msg_id: vote.msg_id as u64,
        session_id: vote.session_id as u64,
        initiator_id: vote.initiator_id as u64,
        yes_count: vote.yes_count as u32,
        no_count: vote.no_count as u32,
        eligible_count: vote.eligible_count as u32,
        deadline: Some(vote.deadline.into()),
        settled: vote.settled,
        passed: vote.passed,
    }
}

fn vote_response(vote: &recall_votes::Model) -> GetRecallVoteResponse {
    GetRecallVoteResponse {
        vote_id: vote.id as u64,
        msg_id: vote.msg_id as u64,
        session_id: vote.session_id as u64,
        initiator_id: vote.initiator_id as u64,
        yes_count: vote.yes_count as u32,
        no_count: vote.no_count as u32,
        eligible_count: vote.eligible_count as u32,
        deadline: Some(vote.deadline.into()),
        settled: vote.settled,
        passed: vote.passed,
    }
}

/// Persist the vote state as a session event and push it to every session
/// member, so both online and offline members see every vote change.
async fn broadcast_vote_state(
    server: &RpcServer,
    vote: &recall_votes::Model,
    actor: Option<ID>,
) -> Result<(), VoteErr> {
    let notification = RespondEventType::RecallVoteNotification(vote_notification(vote));
    let connection = server
        .get_rabbitmq_manager()
        .await
        .context("cannot get rabbitmq connection")?;
    let mut channel = connection
        .create_channel()
        .await
        .context("cannot create channel")?;
    let session_id = SessionID::from(vote.session_id);
    message_insert_and_transmit(
        actor,
        Some(session_id),
        notification,
        Dest::Session(session_id),
        false,
        &server.db.db_pool,
        &mut channel,
    )
    .await?;
    Ok(())
}

/// Lazily settle an expired vote as failed.
///
/// Every entry point must call this before acting on a vote. Returns the
/// up-to-date vote row and whether this call settled it as expired.
async fn settle_if_expired(
    server: &RpcServer,
    vote: &recall_votes::Model,
    actor: Option<ID>,
) -> Result<(recall_votes::Model, bool), VoteErr> {
    if vote.settled || vote.deadline.with_timezone(&chrono::Utc) >= chrono::Utc::now() {
        return Ok((vote.clone(), false));
    }
    let update = settle_vote(vote.id as u64, false, &server.db.db_pool).await?;
    if update.settled_now {
        broadcast_vote_state(server, &update.vote, actor).await?;
    }
    Ok((update.vote, update.settled_now))
}

/// Whether the vote reached a passing majority (strictly more than half of
/// the eligible members approved).
fn vote_passed(vote: &recall_votes::Model) -> bool {
    vote.yes_count * 2 > vote.eligible_count
}

/// Whether the vote can no longer reach a majority (at least half of the
/// eligible members rejected), so it can be settled as failed early.
fn vote_doomed(vote: &recall_votes::Model) -> bool {
    vote.no_count * 2 >= vote.eligible_count
}

async fn start_recall_vote_internal(
    server: &RpcServer,
    id: ID,
    request: Request<StartRecallVoteRequest>,
) -> Result<StartRecallVoteResponse, VoteErr> {
    let req = request.into_inner();
    let session_id = SessionID(req.session_id);
    let db_conn = &server.db.db_pool;

    if !in_session(id, session_id, db_conn).await? {
        return Err(Status::permission_denied(NOT_IN_SESSION).into());
    }
    // A member who may recall on their own has no business starting a vote
    if if_permission_exist(
        id,
        session_id,
        PredefinedPermissions::RecallMsg.into(),
        db_conn,
    )
    .await?
    {
        return Err(Status::invalid_argument(RECALL_PERMISSION_HELD).into());
    }
    // The message must exist (recalled messages are deleted, so existence
    // implies "not yet recalled"), belong to the session and be a real user
    // message instead of a system event
    let msg = get_msg_by_id(req.msg_id, db_conn).await?;
    if msg.session_id != Some(session_id.into()) {
        return Err(Status::not_found(not_found::MSG).into());
    }
    if msg.sender_id == Some(id.into()) {
        // the sender should recall their own message directly
        return Err(Status::invalid_argument(CANNOT_RECALL_BY_VOTE).into());
    }
    let event: RespondEventType =
        serde_json::from_value(msg.msg_data).map_err(|e| VoteErr::Unknown(e.into()))?;
    if !matches!(event, RespondEventType::Msg(_)) {
        // system events (recall events, invitations, ...) cannot be recalled
        return Err(Status::invalid_argument(CANNOT_RECALL_BY_VOTE).into());
    }
    // Only one open vote per message
    if get_open_vote_by_msg_id(req.msg_id, db_conn)
        .await?
        .is_some()
    {
        return Err(Status::already_exists(exist::UNSETTLED_RECALL_VOTE).into());
    }

    // Eligible voters: every session member except the sender of the message
    let sender_id = msg.sender_id;
    let members = get_members(session_id, db_conn).await?;
    let eligible_count = members
        .iter()
        .filter(|m| Some(m.user_id) != sender_id)
        .count() as u32;

    let duration = server.shared_data.cfg().main_cfg.recall_vote_duration;
    let deadline = (chrono::Utc::now() + duration).into();
    let vote = insert_vote(
        req.msg_id,
        session_id,
        id,
        eligible_count,
        deadline,
        db_conn,
    )
    .await?;
    broadcast_vote_state(server, &vote, Some(id)).await?;
    Ok(StartRecallVoteResponse {
        vote_id: vote.id as u64,
    })
}

async fn vote_recall_internal(
    server: &RpcServer,
    id: ID,
    request: Request<VoteRecallRequest>,
) -> Result<VoteRecallResponse, VoteErr> {
    let req = request.into_inner();
    let db_conn = &server.db.db_pool;

    let vote = get_vote_by_id(req.vote_id, db_conn)
        .await?
        .ok_or_else(|| Status::not_found(not_found::RECALL_VOTE))?;
    let session_id = SessionID::from(vote.session_id);
    if !in_session(id, session_id, db_conn).await? {
        return Err(Status::permission_denied(NOT_IN_SESSION).into());
    }
    // The sender of the message is not an eligible voter. If the message is
    // already gone (recalled through another path), the vote simply runs to
    // its normal settlement.
    match get_msg_by_id(vote.msg_id as u64, db_conn).await {
        Ok(msg) if msg.sender_id == Some(id.into()) => {
            return Err(Status::permission_denied(PERMISSION_DENIED).into());
        }
        Ok(_) => {}
        Err(MsgError::NotFound) => {}
        Err(e) => return Err(e.into()),
    }
    // Expired votes are settled as failed first, then rejected
    let (vote, expired_now) = settle_if_expired(server, &vote, Some(id)).await?;
    if vote.settled {
        return Err(Status::failed_precondition(if expired_now {
            VOTE_EXPIRED
        } else {
            VOTE_SETTLED
        })
        .into());
    }

    let vote = match cast_vote_record(&vote, id, req.approve, db_conn).await {
        Ok(vote) => vote,
        Err(VoteDbError::DuplicateVote) => {
            return Err(Status::already_exists(exist::RECALL_VOTE_RECORD).into());
        }
        Err(e) => return Err(e.into()),
    };

    // Settle as soon as the outcome is decided
    if vote_passed(&vote) {
        let update = settle_vote(vote.id as u64, true, db_conn).await?;
        broadcast_vote_state(server, &update.vote, Some(id)).await?;
        if update.settled_now {
            // the majority vote replaces the RecallMsg permission, so neither
            // the permission check nor the recall time limit apply
            if let Err(e) = super::recall::force_recall(
                server,
                SessionID::from(update.vote.session_id),
                update.vote.msg_id as u64,
                Some(id),
            )
            .await
            {
                // the message may have been recalled through another path
                // (e.g. by an admin) while the vote was open; the vote is
                // settled either way
                tracing::warn!("recall after vote failed: {}", e);
            }
        }
    } else if vote_doomed(&vote) {
        let update = settle_vote(vote.id as u64, false, db_conn).await?;
        broadcast_vote_state(server, &update.vote, Some(id)).await?;
    } else {
        broadcast_vote_state(server, &vote, Some(id)).await?;
    }
    Ok(VoteRecallResponse {})
}

async fn get_recall_vote_internal(
    server: &RpcServer,
    id: ID,
    request: Request<GetRecallVoteRequest>,
) -> Result<GetRecallVoteResponse, VoteErr> {
    let req = request.into_inner();
    let db_conn = &server.db.db_pool;

    let vote = get_vote_by_id(req.vote_id, db_conn)
        .await?
        .ok_or_else(|| Status::not_found(not_found::RECALL_VOTE))?;
    if !in_session(id, SessionID::from(vote.session_id), db_conn).await? {
        return Err(Status::permission_denied(NOT_IN_SESSION).into());
    }
    let (vote, _) = settle_if_expired(server, &vote, Some(id)).await?;
    Ok(vote_response(&vote))
}
