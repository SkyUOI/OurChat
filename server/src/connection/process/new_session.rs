use crate::{
    DbPool, SharedData,
    client::{
        requests::{self, AcceptSession, NewSession},
        response::{InviteSession, NewSessionResponse},
    },
    component::EmailSender,
    connection::NetSender,
    consts::{ID, OCID, SessionID, TimeStamp},
    utils,
};
use anyhow::bail;
use redis::AsyncCommands;
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

fn mapped_to_ocid(key: &str) -> String {
    format!("ocid:{}", key)
}

#[derive::db_compatibility]
pub async fn get_id(ocid: &OCID, db_conn: &DbPool) -> anyhow::Result<ID> {
    use entities::user;
    // first query in redis
    let mut redis_conn = db_conn.redis_pool.get().await?;
    let id: Option<u64> = redis_conn.get(mapped_to_ocid(ocid)).await?;
    if let Some(id) = id {
        return Ok(id.into());
    }
    // query in database
    let user = user::Entity::find()
        .filter(user::Column::Ocid.eq(ocid.to_string()))
        .one(&db_conn.db_pool)
        .await?;
    if let Some(user) = user {
        let id = user.id;
        let _: () = redis_conn.set(mapped_to_ocid(ocid), id).await?;
        return Ok(id.into());
    }
    bail!("ocid not found")
}

#[derive::db_compatibility]
pub async fn whether_to_verify(sender: ID, invitee: ID, db_conn: &DbPool) -> anyhow::Result<bool> {
    use entities::friend;
    use entities::prelude::*;
    let invitee: u64 = invitee.into();
    let friend = Friend::find_by_id(sender)
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
    id: ID,
    net_sender: impl NetSender + Clone,
    json: NewSession,
    db_conn: &DbPool,
    shared_data: &Arc<SharedData<impl EmailSender>>,
) -> anyhow::Result<()> {
    let session_id = utils::GENERATOR.generate()?.into_i64().into();

    // check whether to send verification request
    let mut people_num = 1;
    for i in &json.members {
        let member_id = get_id(i, db_conn).await?;
        let verify = whether_to_verify(id, member_id, db_conn).await?;
        if verify {
            send_verification_request(
                id,
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
                .send(NewSessionResponse::failed(e).into())
                .await?;
            return Ok(());
        }
    } else {
        net_sender
            .send(NewSessionResponse::success(session_id).into())
            .await?;
    }
    Ok(())
}

pub fn mapped_to_operations(id: ID) -> String {
    format!("id-oper:{}", id)
}

pub async fn send_verification_request(
    sender: ID,
    invitee: ID,
    session_id: SessionID,
    message: String,
    shared_data: &Arc<SharedData<impl EmailSender>>,
    db_conn: &DbPool,
) -> anyhow::Result<()> {
    let expiresat = chrono::Utc::now() + Duration::from_days(3);
    let request = InviteSession::new(
        //TODO: Move this to config
        expiresat,
        session_id.to_string(),
        sender.to_string(),
        message,
    );
    // try to find connected client
    match shared_data.connected_clients.get(&invitee) {
        Some(client) => {
            client.send(request.into()).await?;
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
    json: AcceptSession,
    db_conn: &DbPool,
) -> anyhow::Result<()> {
    Ok(())
}
