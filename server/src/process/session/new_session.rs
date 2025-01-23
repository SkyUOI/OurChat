use super::super::basic::{get_id, get_ocid};
use super::super::get_id_from_req;
use crate::process::error_msg::{SERVER_ERROR, not_found};
use crate::{SharedData, server::RpcServer, utils};
use anyhow::Context;
use base::consts::{ID, OCID, SessionID};
use base::database::DbPool;
use base::time::TimeStamp;
use entities::{friend, operations, prelude::*, session, session_relation, user_role_relation};
use migration::m20241229_022701_add_role_for_session::PreDefinedRoles;
use pb::ourchat::session::new_session::v1::{NewSessionRequest, NewSessionResponse};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    QueryFilter,
};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tonic::{Request, Response};
use tracing::error;

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
    #[error("database error:{0:?}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("unknown error:{0:?}")]
    UnknownError(#[from] anyhow::Error),
}

/// create a new session in the database
pub async fn create_session_db(
    session_id: SessionID,
    people_num: usize,
    session_name: String,
    db_conn: &DatabaseConnection,
) -> Result<(), SessionError> {
    let time_now = chrono::Utc::now();
    let session = session::ActiveModel {
        session_id: ActiveValue::Set(session_id.into()),
        name: ActiveValue::Set(session_name),
        size: ActiveValue::Set(people_num.try_into().context("people num error")?),
        created_time: ActiveValue::Set(time_now.into()),
        updated_time: ActiveValue::Set(time_now.into()),
        ..Default::default()
    };
    session.insert(db_conn).await?;
    Ok(())
}

/// check the privilege and whether to send a verification request
pub async fn whether_to_verify(
    sender: ID,
    invitee: ID,
    db_conn: &DbPool,
) -> Result<bool, SessionError> {
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
    req: Request<NewSessionRequest>,
) -> Result<NewSessionResponse, SessionError> {
    let session_id = utils::generate_session_id()?;
    let id = get_id_from_req(&req).unwrap();
    let ocid = match get_ocid(id, &server.db).await {
        Ok(ocid) => ocid,
        Err(_) => {
            return Err(SessionError::UserNotFound);
        }
    };

    let req = req.into_inner();
    // check whether to send a verification request
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
        batch_add_to_session(
            session_id,
            &peoples,
            PreDefinedRoles::Member.into(),
            &server.db.db_pool,
        )
        .await?;

        Ok::<(), SessionError>(())
    };
    bundle.await?;
    Ok(NewSessionResponse {
        session_id: session_id.into(),
    })
}

pub async fn new_session(
    server: &RpcServer,
    req: Request<NewSessionRequest>,
) -> Result<Response<NewSessionResponse>, tonic::Status> {
    match new_session_impl(server, req).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            SessionError::UserNotFound => Err(tonic::Status::not_found(not_found::USER)),
            SessionError::DbError(_) | SessionError::UnknownError(_) => {
                error!("{}", e);
                Err(tonic::Status::internal(SERVER_ERROR))
            }
        },
    }
}

pub async fn send_verification_request(
    sender: OCID,
    invitee: ID,
    session_id: SessionID,
    message: String,
    shared_data: &Arc<SharedData>,
    db_conn: &DbPool,
) -> anyhow::Result<()> {
    let expire_at =
        chrono::Utc::now() + Duration::from_days(shared_data.cfg.main_cfg.verification_expire_days);
    let request = InviteSession::new(expire_at.into(), session_id, sender, message);
    // try to find a connected client
    // save to the database
    save_invitation_to_db(
        invitee,
        serde_json::to_string(&request).unwrap(),
        expire_at.into(),
        db_conn,
    )
    .await?;
    Ok(())
}

async fn save_invitation_to_db(
    id: ID,
    operation: String,
    expire_at: TimeStamp,
    db_conn: &DbPool,
) -> anyhow::Result<()> {
    let operation_model = operations::ActiveModel {
        user_id: ActiveValue::Set(id.into()),
        operation: ActiveValue::Set(operation),
        once: ActiveValue::Set(true),
        expires_at: ActiveValue::Set(expire_at),
        ..Default::default()
    };
    operation_model.insert(&db_conn.db_pool).await?;
    Ok(())
}

pub async fn add_to_session(
    session_id: SessionID,
    id: ID,
    role: u64,
    db_conn: &impl ConnectionTrait,
) -> anyhow::Result<()> {
    let session_relation = session_relation::ActiveModel {
        user_id: ActiveValue::Set(id.into()),
        session_id: ActiveValue::Set(session_id.into()),
        ..Default::default()
    };
    session_relation.insert(db_conn).await?;
    // Add role
    let role_relation = user_role_relation::ActiveModel {
        user_id: ActiveValue::Set(id.into()),
        session_id: ActiveValue::Set(session_id.into()),
        role_id: ActiveValue::Set(role as i64),
    };
    role_relation.insert(db_conn).await?;
    Ok(())
}

pub async fn batch_add_to_session(
    session_id: SessionID,
    ids: &[ID],
    role: u64,
    db_conn: &impl ConnectionTrait,
) -> anyhow::Result<()> {
    for id in ids {
        add_to_session(session_id, *id, role, db_conn).await?;
    }
    Ok(())
}
