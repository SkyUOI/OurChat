use std::{sync::Arc, time::Duration};

use client::TestApp;
use client::oc_helper::server_manager::TestServerManager;
use migration::m20250218_093632_server_manage_permission::PredefinedServerManagementRole;
use pb::service::{
    ourchat::msg_delivery::{
        announcement::v1::{Announcement, AnnouncementResponse},
        v1::fetch_msgs_response::RespondMsgType,
    },
    server_manage::publish_announcement::v1::PublishAnnouncementRequest,
};
use server::process::{add_announcement, get_announcement_by_id, get_announcements_by_time};
use tokio::{join, sync::Mutex, time::sleep};
use tonic::Request;

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
            panic!("add an announcement failed: {e}")
        }
    };
    let announcement_res = get_announcement_by_id(app.get_db_connection(), id)
        .await
        .unwrap();
    assert_eq!(announcement_res.announcement.unwrap(), announcement);
    let mut original_announcement = Vec::new();
    for idx in 0..10 {
        let announcement = Announcement {
            title: format!("test{idx}"),
            content: format!("test{idx}"),
            publisher_id: user.as_ref().lock().await.id.into(),
        };
        original_announcement.push(announcement.clone());
        match add_announcement(app.get_db_connection(), announcement.clone()).await {
            Ok(_) => {}
            Err(e) => {
                panic!("add an announcement failed: {e}")
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
    let user_id = user.lock().await.id;
    tracing::info!("user id: {}", user.lock().await.id);

    let server_manager = Arc::new(Mutex::new(
        TestServerManager::new(&app, user_id, user.lock().await.token.clone())
            .await
            .unwrap(),
    ));
    server_manager
        .lock()
        .await
        .assign_role(PredefinedServerManagementRole::Admin as i64)
        .await
        .unwrap();
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
            .publish_announcement(Request::new(PublishAnnouncementRequest {
                announcement: Some(announcement_bef.clone()),
            }))
            .await
            .unwrap();
        sleep(Duration::from_millis(200)).await;
        let receive = user.lock().await.fetch_msgs().fetch(2).await.unwrap();
        assert_eq!(receive.len(), 2, "{receive:?}");
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
    sleep(Duration::from_millis(400)).await;
    server_manager
        .lock()
        .await
        .client
        .publish_announcement(Request::new(PublishAnnouncementRequest {
            announcement: Some(announcement),
        }))
        .await
        .unwrap();

    join!(task).0.unwrap();
    app.async_drop().await
}
