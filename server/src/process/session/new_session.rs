use crate::db::messages::insert_msg_record;
use crate::db::session::SessionError;
use crate::process::error_msg::{SERVER_ERROR, not_found};
use crate::process::{Dest, check_user_exist, transmit_msg};
use crate::{db, helper, server::RpcServer};
use anyhow::Context;
use base::consts::{ID, SessionID};
use base::database::DbPool;
use entities::{friend, prelude::*};
use invite_session::v1::InviteSession;
use pb::service::ourchat::msg_delivery::v1::FetchMsgsResponse;
use pb::service::ourchat::msg_delivery::v1::fetch_msgs_response::RespondMsgType;
use pb::service::ourchat::session::invite_session;
use pb::service::ourchat::session::new_session::v1::{
    FailedMember, FailedReason, NewSessionRequest, NewSessionResponse,
};
use pb::time::to_google_timestamp;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use tonic::{Request, Response};
use tracing::error;

#[derive(Debug, thiserror::Error)]
pub enum NewSessionError {
    #[error("session not found")]
    SessionNotFound,
    #[error("database error:{0:?}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("unknown error:{0:?}")]
    UnknownError(#[from] anyhow::Error),
}

/// check the privilege and whether to send a verification request
pub async fn whether_to_verify(
    sender: ID,
    invitee: ID,
    db_conn: &DbPool,
) -> Result<bool, NewSessionError> {
    let friend = Friend::find()
        .filter(friend::Column::UserId.eq(sender))
        .filter(friend::Column::FriendId.eq(invitee))
        .one(&db_conn.db_pool)
        .await?;
    if let Some(_friend) = friend {
        // don't need to verify
        Ok(false)
    } else {
        Ok(true)
    }
}

async fn new_session_impl(
    server: &RpcServer,
    id: ID,
    req: Request<NewSessionRequest>,
) -> Result<NewSessionResponse, NewSessionError> {
    let session_id = helper::generate_session_id()?;
    let mut failed_members = vec![];
    let req = req.into_inner();
    // check whether to send a verification request
    let mut people_num = 1;
    let mut peoples = vec![id];
    let mut need_to_verify = vec![];
    for i in &req.members {
        let member_id: ID = (*i).into();
        if !check_user_exist(member_id, &server.db.db_pool).await? {
            failed_members.push(FailedMember {
                id: member_id.into(),
                reason: FailedReason::MemberNotFound.into(),
            });
            continue;
        }
        // ignore self
        if member_id == id {
            continue;
        }
        let verify = whether_to_verify(id, member_id, &server.db).await?;
        if verify {
            need_to_verify.push(member_id);
        } else {
            people_num += 1;
            peoples.push(member_id);
        }
    }
    let bundle = async {
        let transaction = server.db.db_pool.begin().await?;
        db::session::create_session_db(
            session_id,
            people_num,
            req.name.unwrap_or_default(),
            &transaction,
            req.e2ee_on,
        )
        .await?;
        // add session relation
        match db::session::batch_join_in_session(session_id, &peoples, None, &transaction).await {
            Ok(_) => {
                transaction.commit().await?;
            }
            Err(SessionError::SessionNotFound) => {
                transaction.rollback().await?;
                return Err(NewSessionError::SessionNotFound);
            }
            Err(SessionError::Db(e)) => {
                transaction.rollback().await?;
                return Err(NewSessionError::DbError(e));
            }
        }
        Ok::<(), NewSessionError>(())
    };
    bundle.await?;
    for member_id in need_to_verify {
        send_verification_request(server, id, member_id, session_id, req.leave_message.clone())
            .await?;
    }
    Ok(NewSessionResponse {
        session_id: session_id.into(),
        failed_members,
    })
}

pub async fn new_session(
    server: &RpcServer,
    id: ID,
    req: Request<NewSessionRequest>,
) -> Result<Response<NewSessionResponse>, tonic::Status> {
    match new_session_impl(server, id, req).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            NewSessionError::SessionNotFound => Err(tonic::Status::not_found(not_found::SESSION)),
            NewSessionError::DbError(_) | NewSessionError::UnknownError(_) => {
                error!("{}", e);
                Err(tonic::Status::internal(SERVER_ERROR))
            }
        },
    }
}

pub async fn send_verification_request(
    server: &RpcServer,
    sender: ID,
    invitee: ID,
    session_id: SessionID,
    leave_message: String,
) -> anyhow::Result<()> {
    let expire_at = chrono::Utc::now() + server.shared_data.cfg.main_cfg.verification_expire_time;
    let expire_at_google = to_google_timestamp(expire_at);
    // save to the database
    let respond_msg = InviteSession {
        session_id: session_id.into(),
        inviter_id: sender.into(),
        leave_message: leave_message.clone(),
        expire_timestamp: Some(expire_at_google),
    };
    let respond_msg = RespondMsgType::InviteSession(respond_msg);
    let msg_model = insert_msg_record(
        invitee.into(),
        Some(session_id),
        respond_msg.clone(),
        false,
        &server.db.db_pool,
        false,
    )
    .await?;
    // try to send the message directly
    let fetch_response = FetchMsgsResponse {
        msg_id: msg_model.msg_id as u64,
        time: Some(expire_at_google),
        respond_msg_type: Some(respond_msg),
    };
    let rabbitmq_connection = server
        .rabbitmq
        .get()
        .await
        .context("cannot get rabbit connection")?;
    let mut channel = rabbitmq_connection
        .create_channel()
        .await
        .context("cannot create channel")?;
    transmit_msg(
        fetch_response,
        Dest::User(invitee),
        &mut channel,
        &server.db.db_pool,
    )
    .await?;
    Ok(())
}
