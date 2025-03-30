use base::time::to_google_timestamp;
use entities::announcement::{self, Model};
use pb::service::ourchat::msg_delivery::announcement::v1::{Announcement, AnnouncementResponse};
use sea_orm::{ConnectionTrait, EntityTrait, Paginator, PaginatorTrait, QueryOrder, SelectModel};
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
    tracing::info!("crated_at found: {:?}", announcement.created_at);
    let created_at = to_google_timestamp(announcement.created_at.to_utc());
    tracing::info!("crated_at: {:?}", created_at);
    Ok(AnnouncementResponse {
        announcement: Some(Announcement {
            id: announcement.id as u64,
            title: announcement.title,
            content: announcement.content,
            publisher_id: announcement.publisher_id as u64,
        }),
        created_at: Some(created_at),
    })
}

pub async fn get_announcements_by_time(
    dbpool: &impl ConnectionTrait,
    page_size: u64,
) -> Result<Paginator<'_, impl ConnectionTrait, SelectModel<Model>>, GetAnnouncementErr> {
    let paginator = announcement::Entity::find()
        .order_by_desc(announcement::Column::CreatedAt)
        .paginate(dbpool, page_size);

    Ok(paginator)
}
