use std::{sync::Arc, time::Duration};

use anyhow::Context;
use client::TestApp;
use hyper::Uri;
use pb::service::{
    ourchat::msg_delivery::{
        announcement::v1::{Announcement, AnnouncementResponse},
        v1::fetch_msgs_response::RespondMsgType,
    },
    server_manage::{
        publish_announcement::v1::PublishAnnouncementRequest,
        v1::server_manage_service_client::ServerManageServiceClient,
    },
};
use server::process::{add_announcement, get_announcement_by_id, get_announcements_by_time};
use tokio::{join, sync::Mutex, time::sleep};
use tonic::{Request, service::interceptor::InterceptedService, transport::Channel};

#[tokio::test]
async fn add_and_get_announcement() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();
    let announcement = Announcement {
        title: "test".to_string(),
        content: "test".to_string(),
        publisher_id: user.as_ref().lock().await.id.into(),
    };
    let id = match add_announcement(app.get_db_connection(), announcement.clone()).await {
        Ok(announcement) => announcement.id,
        Err(e) => {
            tracing::error!("add announcement failed: {}", e);
            panic!("add announcement failed")
        }
    };
    let announcement_res = get_announcement_by_id(app.get_db_connection(), id)
        .await
        .unwrap();
    assert_eq!(announcement_res.announcement.unwrap(), announcement);
    let mut original_announcement = Vec::new();
    for idx in 0..10 {
        let announcement = Announcement {
            title: format!("test{}", idx),
            content: format!("test{}", idx),
            publisher_id: user.as_ref().lock().await.id.into(),
        };
        original_announcement.push(announcement.clone());
        match add_announcement(app.get_db_connection(), announcement.clone()).await {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("add announcement failed: {}", e);
                panic!("add announcement failed")
            }
        };
    }
    let announcements = get_announcements_by_time(app.get_db_connection(), 10)
        .await
        .unwrap();
    let mut announcements: Vec<AnnouncementResponse> = announcements
        .fetch_page(0)
        .await
        .unwrap()
        .iter()
        .map(|val| val.to_owned().1.unwrap().into())
        .rev()
        .collect();
    assert_eq!(announcements.len(), 10);
    for idx in 0..10 {
        assert_eq!(
            announcements[idx].announcement.take().unwrap(),
            original_announcement[idx]
        );
    }

    app.async_drop().await;
}

#[tokio::test]
async fn publish_and_fetch_announcement() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();
    tracing::info!("user id: {}", user.lock().await.id);
    type ServerManagerClient = ServerManageServiceClient<
        InterceptedService<
            Channel,
            Box<
                dyn Fn(tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status>
                    + Send
                    + Sync,
            >,
        >,
    >;
    struct TestServerManager {
        client: Option<ServerManagerClient>,
    }
    impl TestServerManager {
        pub async fn new(app: &TestApp) -> anyhow::Result<Self> {
            let channel =
                Channel::builder(Uri::from_maybe_shared(app.rpc_url.clone()).context("Uri Error")?)
                    .connect()
                    .await
                    .context("Connect Error")?;
            let client: Option<ServerManagerClient> =
                Some(ServerManageServiceClient::with_interceptor(
                    channel,
                    Box::new(move |req: tonic::Request<()>| Ok(req)),
                ));
            Ok(Self { client })
        }
    }
    let server_manager = Arc::new(Mutex::new(TestServerManager::new(&app).await.unwrap()));
    let announcement = Announcement {
        title: "test".to_string(),
        content: "test".to_string(),
        publisher_id: user.as_ref().lock().await.id.into(),
    };
    let announcement_clone = announcement.clone();
    let server_manager_clone = server_manager.clone();

    let task = tokio::spawn(async move {
        let announcement_bef = Announcement {
            title: "testbef".to_string(),
            content: "testbef".to_string(),
            publisher_id: user.as_ref().lock().await.id.into(),
        };
        server_manager_clone
            .lock()
            .await
            .client
            .as_mut()
            .unwrap()
            .publish_announcement(Request::new(PublishAnnouncementRequest {
                announcement: Some(announcement_bef.clone()),
            }))
            .await
            .unwrap();
        let receive = user
            .lock()
            .await
            .fetch_msgs(Duration::from_millis(1000))
            .await
            .unwrap();
        assert_eq!(receive.len(), 2);
        match receive[0].to_owned().respond_msg_type.unwrap() {
            RespondMsgType::AnnouncementResponse(announcement) => {
                assert_eq!(announcement.announcement.unwrap(), announcement_bef);
            }
            _ => panic!("Wrong message type"),
        }
        match receive[1].to_owned().respond_msg_type.unwrap() {
            RespondMsgType::AnnouncementResponse(received_announcement) => {
                assert_eq!(
                    announcement_clone.to_owned().content,
                    received_announcement.announcement.unwrap().content,
                )
            }
            _ => panic!("Wrong message type"),
        };
    });
    sleep(Duration::from_millis(300)).await;
    server_manager
        .lock()
        .await
        .client
        .as_mut()
        .unwrap()
        .publish_announcement(Request::new(PublishAnnouncementRequest {
            announcement: Some(announcement),
        }))
        .await
        .unwrap();

    join!(task).0.unwrap();
    app.async_drop().await
}
