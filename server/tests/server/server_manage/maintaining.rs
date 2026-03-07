use client::TestApp;
use pb::service::server_manage::set_server_status::v1::{ServerStatus, SetServerStatusRequest};
use server::process::error_msg;

#[tokio::test]
async fn set_server_status_can_exit_maintenance_mode() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();

    user.lock()
        .await
        .promote_to_admin(app.get_db_connection())
        .await
        .unwrap();

    user.lock()
        .await
        .server_manage()
        .set_server_status(tonic::Request::new(SetServerStatusRequest {
            server_status: ServerStatus::Maintaining as i32,
            reason: "Test maintenance".to_string(),
        }))
        .await
        .unwrap();

    assert!(
        app.app_shared.get_maintaining(),
        "Server should be in maintenance mode"
    );

    user.lock()
        .await
        .server_manage()
        .set_server_status(tonic::Request::new(SetServerStatusRequest {
            server_status: ServerStatus::Normal as i32,
            reason: "Exit maintenance".to_string(),
        }))
        .await
        .unwrap();

    assert!(
        !app.app_shared.get_maintaining(),
        "Server should be in normal mode"
    );

    app.async_drop().await;
}

#[tokio::test]
async fn server_manage_endpoints_blocked_during_maintenance() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();

    user.lock()
        .await
        .promote_to_admin(app.get_db_connection())
        .await
        .unwrap();

    user.lock()
        .await
        .server_manage()
        .set_server_status(tonic::Request::new(SetServerStatusRequest {
            server_status: ServerStatus::Maintaining as i32,
            reason: "Test maintenance".to_string(),
        }))
        .await
        .unwrap();

    assert!(
        app.app_shared.get_maintaining(),
        "Server should be in maintenance mode"
    );

    use pb::service::server_manage::config::v1::GetConfigRequest;
    let result = user
        .lock()
        .await
        .server_manage()
        .get_config(tonic::Request::new(GetConfigRequest {}))
        .await;

    let err = result.unwrap_err();
    assert_eq!(err.message(), error_msg::MAINTAINING);
    assert_eq!(err.code(), tonic::Code::Unavailable);

    let result = user
        .lock()
        .await
        .server_manage()
        .set_server_status(tonic::Request::new(SetServerStatusRequest {
            server_status: ServerStatus::Normal as i32,
            reason: "Exit maintenance".to_string(),
        }))
        .await;

    assert!(
        result.is_ok(),
        "SetServerStatus should work during maintenance mode to allow exiting"
    );

    assert!(
        !app.app_shared.get_maintaining(),
        "Server should be in normal mode"
    );

    app.async_drop().await;
}
