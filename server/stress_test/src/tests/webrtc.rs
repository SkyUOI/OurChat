use crate::UsersGroup;
use crate::framework::{Report, run_user_stress_test};
use pb::service::ourchat::webrtc::room::create_room::v1::CreateRoomRequest;

use derive::register_test;

#[register_test("Create WebRTC Room", WithUsers)]
pub async fn test_create_room(users: &UsersGroup, report: &mut Report) {
    run_user_stress_test(
        report,
        "create_room",
        users,
        100,
        100,
        |user, _now, _users| async move {
            user.lock()
                .await
                .oc()
                .create_room(CreateRoomRequest {
                    title: Some(format!("Room {}", rand::random::<u32>())),
                    auto_delete: true,
                    open_join: true,
                })
                .await
                .is_ok()
        },
    )
    .await;
}
