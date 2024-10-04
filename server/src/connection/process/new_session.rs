use crate::{
    DbPool,
    client::{
        requests::{self, AcceptSession, NewSession},
        response::{InviteSession, NewSessionResponse},
    },
    connection::NetSender,
    consts::{ID, OCID, SessionID},
    utils,
};
use anyhow::bail;
use redis::AsyncCommands;
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use snowdon::ClassicLayoutSnowflakeExtension;
use std::time::Duration;

#[derive::db_compatibility]
pub async fn create_session(
    group_name: String,
    db_conn: &DatabaseConnection,
) -> anyhow::Result<Result<SessionID, requests::Status>> {
    use entities::session;
    let session_id = utils::GENERATOR.generate()?.into_i64().try_into()?;
    let session = session::ActiveModel {
        session_id: ActiveValue::Set(session_id),
        group_name: ActiveValue::Set(group_name),
    };
    Ok(Ok(session_id.try_into()?))
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
    let invitee: u64 = invitee.into();
    let friend = friend::Entity::find_by_id(sender)
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
) -> anyhow::Result<()> {
    let session_id = match create_session(json.name, &db_conn.db_pool).await? {
        Ok(d) => d,
        Err(e) => {
            net_sender
                .send(NewSessionResponse::failed(e).into())
                .await?;
            return Ok(());
        }
    };
    // check whether to send verification request
    for i in &json.members {
        let member_id = get_id(i, db_conn).await?;
        let verify = whether_to_verify(id, member_id, db_conn).await?;
        if verify {
            send_verification_request(
                net_sender.clone(),
                id,
                member_id,
                session_id,
                json.message.clone(),
            )
            .await?;
        }
    }
    Ok(())
}

pub async fn send_verification_request(
    net_sender: impl NetSender,
    sender: ID,
    invitee: ID,
    session_id: SessionID,
    message: String,
) -> anyhow::Result<()> {
    let request = InviteSession::new(
        //TODO: Move this to config
        (chrono::Utc::now() + Duration::from_days(3))
            .timestamp()
            .try_into()?,
        session_id.to_string(),
        sender.to_string(),
        message,
    );
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
