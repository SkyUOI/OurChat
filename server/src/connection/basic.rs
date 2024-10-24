use super::NetSender;
use crate::{
    DbPool,
    client::{MsgConvert, response},
    consts::{ID, OCID},
};
use anyhow::bail;
use redis::AsyncCommands;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

pub async fn send_error_msg(sender: impl NetSender, msg: impl Into<String>) -> anyhow::Result<()> {
    let error_resp = response::error_msg::ErrorMsgResponse::new(msg.into());
    sender.send(error_resp.to_msg()).await?;
    Ok(())
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
