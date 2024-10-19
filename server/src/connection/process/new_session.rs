use crate::{
    DbPool, SharedData,
    client::{
        MsgConvert,
        requests::{self, AcceptSessionRequest, NewSessionRequest},
        response::{InviteSession, NewSessionResponse},
    },
    component::EmailSender,
    connection::{NetSender, UserInfo, basic::get_id},
    consts::{ID, OCID, SessionID, TimeStamp},
    utils,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use snowdon::ClassicLayoutSnowflakeExtension;
use std::{sync::Arc, time::Duration};

#[derive::db_compatibility]
pub async fn create_session(
    session_id: SessionID,
    people_num: i32,
    group_name: String,
    db_conn: &DatabaseConnection,
) -> anyhow::Result<Result<(), requests::Status>> {
    use entities::session;
    let session = session::ActiveModel {
        session_id: ActiveValue::Set(session_id.into()),
        group_name: ActiveValue::Set(group_name),
        size: ActiveValue::Set(people_num),
    };
    session.insert(db_conn).await?;
    Ok(Ok(()))
}

#[derive::db_compatibility]
pub async fn whether_to_verify(sender: ID, invitee: ID, db_conn: &DbPool) -> anyhow::Result<bool> {
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
    let session_id = utils::GENERATOR.generate()?.into_i64().into();

    // check whether to send verification request
    let mut people_num = 1;
    for i in &json.members {
        let member_id = get_id(i, db_conn).await?;
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
        }
    }
    if people_num != 1 {
        if let Err(e) = create_session(session_id, people_num, json.name, &db_conn.db_pool).await? {
            net_sender
                .send(NewSessionResponse::failed(e).to_msg())
                .await?;
            return Ok(());
        }
    } else {
        net_sender
            .send(NewSessionResponse::success(session_id).to_msg())
            .await?;
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
        expiresat, session_id, sender, message,
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
                expiresat,
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

pub async fn accept_session(
    id: ID,
    net_sender: impl NetSender,
    json: AcceptSessionRequest,
    db_conn: &DbPool,
) -> anyhow::Result<()> {
    // check if the time is expired
    Ok(())
}
