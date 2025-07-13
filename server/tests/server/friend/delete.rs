use client::TestApp;
use pb::service::ourchat::friends::delete_friend::v1::DeleteFriendRequest;
use server::db::friend::query_friend;
use server::process::error_msg;
use tonic::Code;

/// Tests the successful deletion of an existing friend relationship.
///
/// Steps:
/// 1. Create two users (user1 and user2)
/// 2. Establish a friend relationship between them
/// 3. User1 deletes user2 as a friend
/// 4. Verify the friendship no longer exists in the database
#[tokio::test]
async fn delete_friend_success() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user1 = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();
    let (user1_id, user2_id) = (user1.lock().await.id, user2.lock().await.id);

    // Establish friendship (simplified for test context)
    app.create_friendship(user1_id, user2_id).await.unwrap();

    // Check whether the database has the friendship
    assert!(
        query_friend(user1_id, user2_id, app.get_db_connection())
            .await
            .unwrap()
            .is_some(),
        "Friendship should exist"
    );

    // Delete the friendship
    user1
        .lock()
        .await
        .oc()
        .delete_friend(DeleteFriendRequest {
            friend_id: user2_id.into(),
        })
        .await
        .unwrap();

    // Verify friendship is deleted
    assert!(
        query_friend(user1_id, user2_id, app.get_db_connection())
            .await
            .unwrap()
            .is_none(),
        "Friendship should be deleted"
    );

    app.async_drop().await;
}

/// Tests attempting to delete a non-existent friend relationship.
///
/// Steps:
/// 1. Create two users with no existing relationship
/// 2. Attempt to delete the non-existent friendship
/// 3. Verify the appropriate "not found" error is returned
#[tokio::test]
async fn delete_non_existent_friend() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user1 = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();
    let user2_id = user2.lock().await.id;

    // Attempt to delete non-existent friendship
    let err = user1
        .lock()
        .await
        .oc()
        .delete_friend(DeleteFriendRequest {
            friend_id: user2_id.into(),
        })
        .await
        .unwrap_err();

    // Verify error
    assert_eq!(err.code(), Code::NotFound);
    assert_eq!(err.message(), error_msg::not_found::FRIEND);

    app.async_drop().await;
}
