use base::consts::ID;
use sea_orm::{ConnectionTrait, EntityTrait};

pub async fn query_friend(
    user_id: ID,
    friend_id: ID,
    db_conn: &impl ConnectionTrait,
) -> Result<Option<entities::friend::Model>, sea_orm::DbErr> {
    let ret = entities::friend::Entity::find_by_id((user_id.into(), friend_id.into()))
        .one(db_conn)
        .await?;
    Ok(ret)
}

pub async fn delete_friend(
    user_id: ID,
    friend_id: ID,
    db_conn: &impl ConnectionTrait,
) -> Result<(), sea_orm::DbErr> {
    entities::friend::Entity::delete_by_id((user_id.into(), friend_id.into()))
        .exec(db_conn)
        .await?;
    entities::friend::Entity::delete_by_id((friend_id.into(), user_id.into()))
        .exec(db_conn)
        .await?;
    Ok(())
}
