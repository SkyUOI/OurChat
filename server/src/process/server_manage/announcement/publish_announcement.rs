use anyhow::Context;
use pb::service::{
    ourchat::msg_delivery::{
        announcement::v1::AnnouncementResponse,
        v1::{FetchMsgsResponse, fetch_msgs_response::RespondEventType},
    },
    server_manage::publish_announcement::v1::{
        PublishAnnouncementRequest, PublishAnnouncementResponse,
    },
};

use tonic::{Request, Response, Status};

use crate::{
    process::{Dest, add_announcement, transmit_msg},
    server::ServerManageServiceProvider,
};

use super::{add_announcement::AddAnnouncementErr, get_announcement::GetAnnouncementErr};

#[derive(Debug, thiserror::Error)]
pub enum PublishAnnouncementErr {
    #[error("pool error: {0:?}")]
    Unknown(#[from] anyhow::Error),
    #[error("get announcement error: {0:?}")]
    GetAnnouncementErr(#[from] GetAnnouncementErr),
    #[error("add announcement error: {0:?}")]
    AddAnnouncementErr(#[from] AddAnnouncementErr),
}
pub async fn publish_announcement(
    server: &ServerManageServiceProvider,
    request: Request<PublishAnnouncementRequest>,
) -> Result<Response<PublishAnnouncementResponse>, Status> {
    match publish_announcement_internal(server, request).await {
        Ok(response) => Ok(Response::new(response)),
        Err(err) => {
            tracing::error!("{}", err);
            Err(Status::internal(err.to_string()))
        }
    }
}

async fn publish_announcement_internal(
    server: &ServerManageServiceProvider,
    request: Request<PublishAnnouncementRequest>,
) -> Result<PublishAnnouncementResponse, PublishAnnouncementErr> {
    let announcement: AnnouncementResponse = add_announcement(
        &server.db.db_pool,
        request
            .into_inner()
            .announcement
            .ok_or_else(|| anyhow::anyhow!("announcement is none"))?,
    )
    .await?
    .into();
    let connection = server.get_rabbitmq_manager().await?;
    let mut channel = connection
        .create_channel()
        .await
        .context("cannot create channel")?;
    transmit_msg(
        FetchMsgsResponse {
            msg_id: announcement.id,
            time: announcement.created_at,
            respond_event_type: Some(RespondEventType::AnnouncementResponse(announcement.clone())),
        },
        Dest::All,
        &mut channel,
        &server.db.db_pool,
    )
    .await?;
    Ok(PublishAnnouncementResponse {
        announcement: Some(announcement),
    })
}
