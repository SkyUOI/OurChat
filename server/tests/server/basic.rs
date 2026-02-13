use base::constants::VERSION_SPLIT;
use claims::assert_lt;
use client::TestApp;
use pb::service::basic::preset_user_status::v1::GetPresetUserStatusRequest;
use pb::service::basic::support::v1::{ContactRole, SupportRequest};
use pb::service::basic::v1::GetServerInfoRequest;
use server::process::basic::get_preset_user_status::add_preset_user_status;
use server::process::error_msg::not_found;
use tonic::Request;

#[tokio::test]
async fn get_datetime() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let time1 = app.get_timestamp().await;
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    let time2 = app.get_timestamp().await;
    assert_lt!(time1, time2);
    app.async_drop().await;
}

#[tokio::test]
async fn get_server_info() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let req = app
        .basic_service()
        .get_server_info(GetServerInfoRequest {})
        .await
        .unwrap();
    let req = req.into_inner();
    assert_eq!(0, req.status);
    assert_eq!(req.server_version.unwrap(), *VERSION_SPLIT);
    app.async_drop().await;
}

#[tokio::test]
async fn get_id_through_ocid() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user1 = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();
    let ocid = user1.lock().await.ocid.clone();
    let id = app.get_id(ocid).await.unwrap();
    assert_eq!(id, user1.lock().await.id);
    let id = app.get_id(user2.lock().await.ocid.clone()).await.unwrap();
    assert_eq!(id, user2.lock().await.id);
    let err = app
        .get_id(base::constants::OCID("wrong ocid".to_owned()))
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::NotFound);
    assert_eq!(err.message(), not_found::USER);
    app.async_drop().await;
}

#[tokio::test]
async fn get_support_info() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    // Get support information
    let response = app
        .basic_service()
        .support(SupportRequest {})
        .await
        .unwrap()
        .into_inner();

    // Verify support page URL
    assert_eq!(
        response.support_page,
        app.app_shared
            .cfg()
            .user_setting
            .support_page
            .clone()
            .map(|x| x.to_string())
    );

    // Verify contacts
    let cfg_contacts = app.app_shared.cfg().user_setting.contacts.clone();
    assert_eq!(response.contacts.len(), cfg_contacts.len());

    for (resp_contact, cfg_contact) in response.contacts.iter().zip(cfg_contacts.iter()) {
        // Verify role conversion
        match cfg_contact.role {
            base::setting::ContactRole::Admin => {
                assert_eq!(resp_contact.role, ContactRole::Admin as i32)
            }
            base::setting::ContactRole::Security => {
                assert_eq!(resp_contact.role, ContactRole::Security as i32)
            }
        }

        // Verify optional fields
        assert_eq!(
            resp_contact.email_address,
            cfg_contact.email_address.as_ref().map(|e| e.to_string())
        );
        assert_eq!(resp_contact.ocid, cfg_contact.ocid);
        assert_eq!(resp_contact.phone_number, cfg_contact.phone_number);
    }

    app.async_drop().await;
}

#[tokio::test]
async fn get_preset_user_status() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    add_preset_user_status(app.get_db_connection(), "I am good").await;
    add_preset_user_status(app.get_db_connection(), "I am bad").await;
    let statuses = app
        .basic_service()
        .get_preset_user_status(Request::new(GetPresetUserStatusRequest {}))
        .await
        .unwrap()
        .into_inner();
    let statuses: Vec<String> = statuses.contents;
    assert_eq!(statuses.len(), 2);
    assert!(statuses.contains(&"I am good".to_string()));
    assert!(statuses.contains(&"I am bad".to_string()));
    app.async_drop().await;
}
