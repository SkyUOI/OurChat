use crate::{
    DbPool, SharedData,
    client::{
        MsgConvert,
        requests::{self, AcceptSessionRequest, NewSessionRequest},
        response::{ErrorMsgResponse, InviteSession, NewSessionResponse},
    },
    component::EmailSender,
    connection::{NetSender, UserInfo, basic::get_id},
    consts::{ID, OCID, SessionID},
    utils,
};
use anyhow::Context;
use base::time::TimeStamp;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use std::{sync::Arc, time::Duration};

#[derive(Debug, thiserror::Error)]
pub enum ErrorOfSession {
    #[error("database error")]
    DbError(#[from] sea_orm::DbErr),
    #[error("unknown error")]
    UnknownError(#[from] anyhow::Error),
}

#[derive::db_compatibility]
pub async fn create_session(
    session_id: SessionID,
    people_num: usize,
    session_name: String,
    db_conn: &DatabaseConnection,
) -> Result<Result<(), requests::Status>, ErrorOfSession> {
    use entities::session;
    let session = session::ActiveModel {
        session_id: ActiveValue::Set(session_id.into()),
        name: ActiveValue::Set(session_name),
        size: ActiveValue::Set(people_num.try_into().context("people num error")?),
    };
    session.insert(db_conn).await?;
    Ok(Ok(()))
}

#[derive::db_compatibility]
pub async fn whether_to_verify(
    sender: ID,
    invitee: ID,
    db_conn: &DbPool,
) -> Result<bool, ErrorOfSession> {
    use entities::friend;
    use entities::prelude::*;
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

pub async fn new_session(
    user_info: &UserInfo,
    net_sender: impl NetSender + Clone,
    json: NewSessionRequest,
    db_conn: &DbPool,
    shared_data: &Arc<SharedData<impl EmailSender>>,
) -> anyhow::Result<()> {
    let session_id = utils::generate_session_id()?;

    // check whether to send verification request
    let mut people_num = 1;
    let mut peoples = vec![user_info.id];
    for i in &json.members {
        let member_id = get_id(i, db_conn).await?;
        // ignore self
        if member_id == user_info.id {
            continue;
        }
        let verify = whether_to_verify(user_info.id, member_id, db_conn).await?;
        if verify {
            send_verification_request(
                user_info.ocid.clone(),
                member_id,
                session_id,
                json.message.clone(),
                shared_data,
                db_conn,
            )
            .await?;
        } else {
            people_num += 1;
            peoples.push(member_id);
        }
    }
    let bundle = async {
        if let Err(e) = create_session(session_id, people_num, json.name, &db_conn.db_pool).await? {
            net_sender
                .send(ErrorMsgResponse::server_error("Database error").to_msg())
                .await?;
            tracing::error!("create session error: {}", e);
        }
        // add session relation
        batch_add_to_session(&db_conn.db_pool, session_id, &peoples).await?;

        anyhow::Ok(())
    };
    match bundle.await {
        Ok(_) => {
            net_sender
                .send(NewSessionResponse::success(session_id).to_msg())
                .await?;
        }
        Err(e) => {
            tracing::error!("{e}");
        }
    }

    Ok(())
}

pub async fn send_verification_request(
    sender: OCID,
    invitee: ID,
    session_id: SessionID,
    message: String,
    shared_data: &Arc<SharedData<impl EmailSender>>,
    db_conn: &DbPool,
) -> anyhow::Result<()> {
    let expiresat = chrono::Utc::now() + Duration::from_days(3);
    let request = InviteSession::new(
        //TODO: Move this to config
        expiresat.into(),
        session_id,
        sender,
        message,
    );
    // try to find connected client
    match shared_data.connected_clients.get(&invitee) {
        Some(client) => {
            client.send(request.to_msg()).await?;
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

#[derive::db_compatibility]
async fn save_invitation_to_db(
    id: ID,
    operation: String,
    expiresat: TimeStamp,
    db_conn: &DbPool,
) -> anyhow::Result<()> {
    use entities::operations;
    let oper = operations::ActiveModel {
        id: ActiveValue::Set(id.into()),
        operation: ActiveValue::Set(operation),
        once: ActiveValue::Set(true.into()),
        expires_at: ActiveValue::Set(expiresat),
        ..Default::default()
    };
    oper.insert(&db_conn.db_pool).await?;
    Ok(())
}

#[derive::db_compatibility]
pub async fn add_to_session(
    db_conn: &DatabaseConnection,
    session_id: SessionID,
    id: ID,
) -> anyhow::Result<()> {
    use entities::prelude::*;
    use entities::session_relation;
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

pub async fn accept_session(
    id: ID,
    net_sender: impl NetSender,
    json: AcceptSessionRequest,
    db_conn: &DbPool,
) -> anyhow::Result<()> {
    // check if the time is expired
    Ok(())
}
