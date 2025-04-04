use entities::{announcement, prelude::*};
use pb::service::ourchat::msg_delivery::announcement::v1::AnnouncementResponse;
use sea_orm::{
    ConnectionTrait, EntityTrait, Paginator, PaginatorTrait, QueryOrder, SelectTwoModel,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetAnnouncementErr {
    #[error("Database error: {0:?}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("Announcement not found")]
    NotFound,
}

pub async fn get_announcement_by_id(
    dbpool: &impl ConnectionTrait,
    id: i64,
) -> Result<AnnouncementResponse, GetAnnouncementErr> {
    let announcement = announcement::Entity::find_by_id(id)
        .one(dbpool)
        .await?
        .ok_or(GetAnnouncementErr::NotFound)?;

    Ok(announcement.to_owned().into())
}

pub async fn get_announcements_by_time(
    db_pool: &impl ConnectionTrait,
    page_size: u64,
) -> Result<
    Paginator<
        '_,
        impl ConnectionTrait,
        SelectTwoModel<entities::announcement_msg::Model, entities::announcement::Model>,
    >,
    GetAnnouncementErr,
> {
    let paginator = AnnouncementMsg::find()
        .find_also_related(Announcement)
        .order_by_desc(announcement::Column::CreatedAt)
        .paginate(db_pool, page_size);

    Ok(paginator)
}
