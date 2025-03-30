use entities::announcement;
use pb::service::ourchat::msg_delivery::announcement::v1::Announcement;
use sea_orm::{ActiveModelTrait, ActiveValue, ConnectionTrait};

#[derive(Debug, thiserror::Error)]
pub enum AddAnnouncementErr {
    #[error("Database error: {0:?}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("Time error: {0}")]
    TimeErr(String),
}

pub async fn add_announcement(
    dbpool: &impl ConnectionTrait,
    announcement: Announcement,
) -> Result<(), AddAnnouncementErr> {
    tracing::trace!("Adding announcement: {:?}", announcement);

    let active_model = announcement::ActiveModel {
        id: ActiveValue::Set(announcement.id as i64),
        title: ActiveValue::Set(announcement.title),
        content: ActiveValue::Set(announcement.content),
        publisher_id: ActiveValue::Set(announcement.publisher_id as i64),
        ..Default::default()
    };
    tracing::trace!("Announcement starts to be added");
    active_model.insert(dbpool).await?;
    tracing::trace!("Announcement added successfully");

    Ok(())
}
