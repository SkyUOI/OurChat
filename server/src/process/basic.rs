pub mod support;

use anyhow::bail;
use base::consts::{ID, OCID};
use base::database::DbPool;
use deadpool_redis::redis::AsyncCommands;
use entities::{prelude::*, user};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

fn mapped_to_ocid(key: &str) -> String {
    format!("ocid:{}", key)
}

pub async fn get_id(ocid: &OCID, db_conn: &DbPool) -> anyhow::Result<ID> {
    // first query in redis
    let mut redis_conn = db_conn.redis_pool.get().await?;
    let id: Option<u64> = redis_conn.get(mapped_to_ocid(ocid)).await?;
    if let Some(id) = id {
        return Ok(id.into());
    }
    // query in database
    let user = User::find()
        .filter(user::Column::Ocid.eq(ocid))
        .one(&db_conn.db_pool)
        .await?;
    if let Some(user) = user {
        let id = user.id;
        let _: () = redis_conn.set(mapped_to_ocid(ocid), id).await?;
        return Ok(id.into());
    }
    bail!("ocid not found")
}

pub async fn get_ocid(id: ID, db_conn: &DbPool) -> anyhow::Result<OCID> {
    let user = User::find_by_id(id).one(&db_conn.db_pool).await?;
    if let Some(user) = user {
        return Ok(OCID(user.ocid));
    }
    bail!("id not found")
}
