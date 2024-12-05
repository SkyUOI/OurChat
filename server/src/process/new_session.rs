use super::basic::{get_id, get_ocid};
use crate::{
    DbPool, SharedData,
    component::EmailSender,
    consts::{ID, OCID, SessionID},
    server::RpcServer,
    utils,
};
use anyhow::Context;
use base::time::TimeStamp;
use entities::{friend, operations, prelude::*, session, session_relation};
use pb::ourchat::session::new_session::v1::{NewSessionRequest, NewSessionResponse};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tonic::{Request, Response};
use tracing::error;

use super::get_id_from_req;

#[derive(Debug, Serialize, Deserialize)]
pub struct InviteSession {
    pub expire_timestamp: TimeStamp,
    pub session_id: SessionID,
    pub inviter_id: String,
    pub message: String,
}

impl InviteSession {
    pub fn new(
        expire_timestamp: TimeStamp,
        session_id: SessionID,
        inviter_id: String,
        message: String,
    ) -> Self {
        Self {
            expire_timestamp,
            session_id,
            inviter_id,
            message,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("user not found")]
    UserNotFound,
    #[error("database error")]
    DbError(#[from] sea_orm::DbErr),
    #[error("unknown error")]
    UnknownError(#[from] anyhow::Error),
}

/// create a new session in database
pub async fn create_session_db(
    session_id: SessionID,
    people_num: usize,
    session_name: String,
    db_conn: &DatabaseConnection,
) -> Result<(), SessionError> {
    let session = session::ActiveModel {
        session_id: ActiveValue::Set(session_id.into()),
        name: ActiveValue::Set(session_name),
        size: ActiveValue::Set(people_num.try_into().context("people num error")?),
    };
    session.insert(db_conn).await?;
    Ok(())
}

/// check the privilege and whether to send verification request
pub async fn whether_to_verify(
    sender: ID,
    invitee: ID,
    db_conn: &DbPool,
) -> Result<bool, SessionError> {
    let sender: u64 = sender.into();
    let invitee: u64 = invitee.into();
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
    server: &RpcServer<impl EmailSender>,
    req: Request<NewSessionRequest>,
) -> Result<NewSessionResponse, SessionError> {
    let session_id = utils::generate_session_id()?;
    let id = get_id_from_req(&req).unwrap();
    let ocid = match get_ocid(id, &server.db).await {
        Ok(ocid) => ocid,
        Err(e) => {
            return Err(SessionError::UserNotFound);
        }
    };

    let req = req.into_inner();
    // check whether to send verification request
    let mut people_num = 1;
    let mut peoples = vec![id];
    for i in &req.members {
        let member_id = get_id(i, &server.db).await?;
        // ignore self
        if member_id == id {
            continue;
        }
        let verify = whether_to_verify(id, member_id, &server.db).await?;
        if verify {
            send_verification_request(
                ocid.clone(),
                member_id,
                session_id,
                req.message.clone(),
                &server.shared_data,
                &server.db,
            )
            .await?;
        } else {
            people_num += 1;
            peoples.push(member_id);
        }
    }
    let bundle = async {
        create_session_db(
            session_id,
            people_num,
            req.name.unwrap_or_default(),
            &server.db.db_pool,
        )
        .await?;
        // add session relation
        batch_add_to_session(&server.db.db_pool, session_id, &peoples).await?;

        Ok::<(), SessionError>(())
    };
    bundle.await?;
    Ok(NewSessionResponse {
        session_id: session_id.into(),
    })
}

pub async fn new_session(
    server: &RpcServer<impl EmailSender>,
    req: Request<NewSessionRequest>,
) -> Result<Response<NewSessionResponse>, tonic::Status> {
    match new_session_impl(server, req).await {
        Ok(res) => Ok(Response::new(res)),
        Err(SessionError::UserNotFound) => Err(tonic::Status::not_found("User not found")),
        Err(SessionError::DbError(e)) => {
            error!("{}", e);
            Err(tonic::Status::internal("Database error"))
        }
        Err(SessionError::UnknownError(e)) => {
            error!("{}", e);
            Err(tonic::Status::internal("Unknown error"))
        }
    }
}

pub async fn send_verification_request(
    sender: OCID,
    invitee: ID,
    session_id: SessionID,
    message: String,
    shared_data: &Arc<SharedData<impl EmailSender>>,
    db_conn: &DbPool,
) -> anyhow::Result<()> {
    let expiresat =
        chrono::Utc::now() + Duration::from_days(shared_data.cfg.main_cfg.verification_expire_days);
    let request = InviteSession::new(expiresat.into(), session_id, sender, message);
    // try to find connected client
    match shared_data.connected_clients.get(&invitee) {
        Some(client) => {
            // client.send(request.to_msg()).await?;
            return Ok(());
        }
        None => {
            // save to database
            save_invitation_to_db(
                invitee,
                serde_json::to_string(&request).unwrap(),
                expiresat.into(),
                db_conn,
            )
            .await?;
        }
    }
    Ok(())
}

async fn save_invitation_to_db(
    id: ID,
    operation: String,
    expiresat: TimeStamp,
    db_conn: &DbPool,
) -> anyhow::Result<()> {
    let oper = operations::ActiveModel {
        user_id: ActiveValue::Set(id.into()),
        operation: ActiveValue::Set(operation),
        once: ActiveValue::Set(true),
        expires_at: ActiveValue::Set(expiresat),
        ..Default::default()
    };
    oper.insert(&db_conn.db_pool).await?;
    Ok(())
}

pub async fn add_to_session(
    db_conn: &DatabaseConnection,
    session_id: SessionID,
    id: ID,
) -> anyhow::Result<()> {
    let session_relation = session_relation::ActiveModel {
        user_id: ActiveValue::Set(id.into()),
        session_id: ActiveValue::Set(session_id.into()),
        ..Default::default()
    };
    session_relation.insert(db_conn).await?;
    Ok(())
}

pub async fn batch_add_to_session(
    db_conn: &DatabaseConnection,
    session_id: SessionID,
    ids: &[ID],
) -> anyhow::Result<()> {
    for id in ids {
        add_to_session(db_conn, session_id, *id).await?;
    }
    Ok(())
}
