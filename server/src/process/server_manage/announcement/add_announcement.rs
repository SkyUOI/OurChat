use base::consts::ID;
use entities::{announcement, announcement_msg};
use pb::service::ourchat::msg_delivery::{
    announcement::v1::Announcement, v1::fetch_msgs_response::RespondEventType,
};
use sea_orm::{ActiveModelTrait, ActiveValue, ConnectionTrait};

use crate::db::messages::insert_msg_record;

#[derive(Debug, thiserror::Error)]
pub enum AddAnnouncementErr {
    #[error("Database error: {0:?}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("Time error: {0}")]
    TimeErr(String),
    #[error("Message error: {0}")]
    MessageErr(#[from] crate::db::messages::MsgError),
}

/// Add an announcement to the database and return the announcement
pub async fn add_announcement(
    dbpool: &impl ConnectionTrait,
    announcement: Announcement,
) -> Result<announcement::Model, AddAnnouncementErr> {
    tracing::trace!("Adding announcement: {:?}", announcement);

    let active_model = announcement::ActiveModel {
        title: ActiveValue::Set(announcement.title),
        content: ActiveValue::Set(announcement.content),
        publisher_id: ActiveValue::Set(announcement.publisher_id as i64),
        ..Default::default()
    };

    tracing::trace!("Announcement starts to be added");
    let res = active_model.insert(dbpool).await?;
    tracing::trace!("Announcement added successfully");

    let msg = insert_msg_record(
        ID::from(res.publisher_id).into(),
        None,
        RespondEventType::AnnouncementResponse(res.clone().into()),
        false,
        dbpool,
        true,
    )
    .await?;

    let active_model = announcement_msg::ActiveModel {
        announcement_id: ActiveValue::Set(res.id),
        msg_id: ActiveValue::Set(msg.msg_id),
    };

    active_model.insert(dbpool).await?;

    Ok(res)
}
