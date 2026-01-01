pub mod get_preset_user_status;
pub mod support;

use anyhow::bail;
use base::consts::{ID, OCID};
use base::database::DbPool;
use entities::{prelude::*, user};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

pub async fn get_id(ocid: &OCID, db_conn: &DbPool) -> anyhow::Result<ID> {
    let user = User::find()
        .filter(user::Column::Ocid.eq(ocid))
        .one(&db_conn.db_pool)
        .await?;
    if let Some(user) = user {
        let id = user.id;
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
