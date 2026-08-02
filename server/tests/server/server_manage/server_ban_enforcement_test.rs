//! Tests that a server-wide ban (ServerManageService.BanUser) is actually
//! enforced (C-3). Previously the ban was written to Redis but never read back,
//! so a banned user could keep using all OurChatService endpoints. The hardened
//! `check_account_status` now consults the `server_ban:*` Redis key, so any
//! OurChatService call from a banned user must fail with `PermissionDenied`
//! until an unban removes the key.

use claims::{assert_err, assert_ok};
use client::TestApp;
use pb::google::protobuf::Duration;
use pb::service::ourchat::get_account_info::v1::{GetAccountInfoRequest, QueryValues};
use pb::service::server_manage::user_manage::v1::{BanUserRequest, UnbanUserRequest};
use tonic::{Code, Request};

/// Call OurChatService.GetAccountInfo for the target user (this hits
/// `check_account_status`, which enforces server bans). Returns the raw gRPC
/// status so the caller can inspect the code.
async fn account_info_status(
    user: &client::oc_helper::user::TestUserShared,
) -> Result<(), tonic::Status> {
    let id = user.lock().await.id;
    user.lock()
        .await
        .oc()
        .get_account_info(GetAccountInfoRequest {
            id: Some(id.into()),
            request_values: vec![QueryValues::UpdatedTime.into()],
        })
        .await
        .map(|_| ())
}

#[tokio::test]
async fn server_ban_blocks_account_and_unban_restores() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    let admin = app.new_user().await.unwrap();
    let target = app.new_user().await.unwrap();
    let target_id = target.lock().await.id;

    admin
        .lock()
        .await
        .promote_to_admin(app.get_db_connection())
        .await
        .unwrap();

    // Before the ban, the target can read account info.
    assert_ok!(account_info_status(&target).await);

    // Admin bans the target for 5 minutes.
    assert_ok!(
        admin
            .lock()
            .await
            .server_manage()
            .ban_user(Request::new(BanUserRequest {
                user_id: target_id.into(),
                reason: Some("abuse".to_string()),
                duration: Some(Duration {
                    seconds: 300,
                    nanos: 0
                }),
            }))
            .await
    );

    // Give Redis a brief moment to propagate the write.
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Now every OurChatService call by the target must be rejected. The JWT is
    // still valid (it has not expired); the rejection comes from
    // check_account_status reading the server_ban key.
    let err = account_info_status(&target)
        .await
        .expect_err("banned user must be rejected");
    assert_eq!(
        err.code(),
        Code::PermissionDenied,
        "banned user must be rejected with PermissionDenied, got {err:?}"
    );

    // Admin unbans the target.
    assert_ok!(
        admin
            .lock()
            .await
            .server_manage()
            .unban_user(Request::new(UnbanUserRequest {
                user_id: target_id.into(),
            }))
            .await
    );

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // After unban, the target can act again.
    assert_ok!(account_info_status(&target).await);

    app.async_drop().await;
}

#[tokio::test]
async fn permanent_server_ban_persists_until_unban() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    let admin = app.new_user().await.unwrap();
    let target = app.new_user().await.unwrap();
    let target_id = target.lock().await.id;

    admin
        .lock()
        .await
        .promote_to_admin(app.get_db_connection())
        .await
        .unwrap();

    // Permanent ban (no duration).
    assert_ok!(
        admin
            .lock()
            .await
            .server_manage()
            .ban_user(Request::new(BanUserRequest {
                user_id: target_id.into(),
                reason: None,
                duration: None,
            }))
            .await
    );

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Still banned.
    assert_err!(account_info_status(&target).await);

    // Unban clears it.
    assert_ok!(
        admin
            .lock()
            .await
            .server_manage()
            .unban_user(Request::new(UnbanUserRequest {
                user_id: target_id.into(),
            }))
            .await
    );

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    assert_ok!(account_info_status(&target).await);

    app.async_drop().await;
}
