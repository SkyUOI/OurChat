mod ban;
mod delete;
mod e2ee_update;
mod e2eeize_and_dee2eeize_session;
mod invite;
mod join;
mod leave;
mod mute;
mod role;

use base::consts::{ID, SessionID};
use base::types::RoleId;
use claims::{assert_lt, assert_ok};
use client::TestApp;
use migration::m20241229_022701_add_role_for_session::PredefinedRoles;
use parking_lot::Mutex;
use pb::service::ourchat::msg_delivery::v1::FetchMsgsResponse;
use pb::service::ourchat::msg_delivery::v1::fetch_msgs_response::RespondEventType;
use pb::service::ourchat::session::{
    get_session_info::v1::{GetSessionInfoRequest, QueryValues},
    new_session::v1::NewSessionRequest,
    set_session_info::v1::SetSessionInfoRequest,
};
use pb::time::TimeStampUtc;
use server::db::session::get_all_session_relations;
use server::process::error_msg;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Notify, oneshot};

#[tokio::test]
async fn session_create() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user1 = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();
    let user3 = app.new_user().await.unwrap();
    let (user1_id, user2_id, user3_id) = (
        user1.lock().await.id,
        user2.lock().await.id,
        user3.lock().await.id,
    );

    let user2_rec = Arc::new(Mutex::new(None));
    let user2_rec_clone = user2_rec.clone();
    let user2_clone = user2.clone();
    let notify = Arc::new(Notify::new());
    let notify_clone = notify.clone();

    let (tx, rx) = oneshot::channel();
    let task = tokio::spawn(async move {
        tx.send(()).unwrap();
        let ret = user2_clone
            .lock()
            .await
            .fetch_msgs()
            .fetch_with_notify(notify_clone)
            .await
            .unwrap();
        *user2_rec_clone.lock() = Some(ret);
    });
    // try to create a session in two users
    let req = NewSessionRequest {
        members: vec![user2_id.into(), user3_id.into()],
        leave_message: Some("hello".to_string()),
        ..Default::default()
    };
    // wait for user2 to listen
    rx.await.unwrap();
    // get new session response
    let ret = user1.lock().await.oc().new_session(req).await.unwrap();
    let new_session = ret.into_inner();
    let session_id: SessionID = new_session.session_id.into();
    assert_eq!(new_session.failed_members, vec![]);
    let user3_rec = user3.lock().await.fetch_msgs().fetch(1).await.unwrap();
    let check = async |rec: Vec<FetchMsgsResponse>| {
        assert_eq!(rec.len(), 1);
        let RespondEventType::InviteUserToSession(rec) = rec[0].respond_event_type.clone().unwrap()
        else {
            panic!();
        };
        assert_eq!(rec.session_id, *session_id);
        assert_eq!(rec.inviter_id, *user1_id);
        assert_eq!(rec.leave_message, Some("hello".to_string()));
    };
    check(user3_rec).await;
    notify.notify_waiters();
    tokio::join!(task).0.unwrap();
    let rec = user2_rec.lock().clone();
    check(rec.unwrap()).await;
    // user2 reject, user3 accept
    user2
        .lock()
        .await
        .accept_join_session_invitation(session_id, false, user1_id)
        .await
        .unwrap();
    user3
        .lock()
        .await
        .accept_join_session_invitation(session_id, true, user1_id)
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;
    let err = user2
        .lock()
        .await
        .accept_join_session_invitation(session_id, false, user1_id)
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::NotFound);
    assert_eq!(err.message(), error_msg::not_found::SESSION_INVITATION);
    let err = user3
        .lock()
        .await
        .accept_join_session_invitation(session_id, true, user1_id)
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::NotFound);
    assert_eq!(err.message(), error_msg::not_found::SESSION_INVITATION);

    assert_eq!(
        get_all_session_relations(user2_id, app.get_db_connection())
            .await
            .unwrap(),
        vec![]
    );
    assert_eq!(
        get_all_session_relations(user3_id, app.get_db_connection())
            .await
            .unwrap()
            .len(),
        1
    );
    // try set info
    let request = SetSessionInfoRequest {
        session_id: session_id.into(),
        name: Some("test name".to_owned()),
        description: Some("test description".to_owned()),
        avatar_key: Some("pic key".to_owned()),
    };
    // accept request from owner
    user1
        .lock()
        .await
        .oc()
        .set_session_info(request.clone())
        .await
        .unwrap();
    // reject request from non-owner
    let err = user3
        .lock()
        .await
        .oc()
        .set_session_info(request)
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
    assert_eq!(err.message(), error_msg::CANNOT_SET_NAME);
    app.async_drop().await;
}

#[tokio::test]
async fn get_session_info() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app
        .new_session_db_level(3, "session1", false)
        .await
        .unwrap();
    let (a, b, c) = (
        session_user[0].clone(),
        session_user[1].clone(),
        session_user[2].clone(),
    );
    let info = a
        .lock()
        .await
        .oc()
        .get_session_info(GetSessionInfoRequest {
            session_id: session.session_id.into(),
            query_values: vec![
                QueryValues::Size.into(),
                QueryValues::Name.into(),
                QueryValues::Unspecified.into(),
                QueryValues::AvatarKey.into(),
                QueryValues::CreatedTime.into(),
                QueryValues::SessionId.into(),
                QueryValues::Members.into(),
                QueryValues::Roles.into(),
            ],
        })
        .await
        .unwrap();
    let time_now = app.get_timestamp().await;
    let info = info.into_inner();
    assert_eq!(info.name.unwrap(), "session1");
    assert_eq!(info.size.unwrap(), 3);
    let tmp: TimeStampUtc = info.created_time.unwrap().try_into().unwrap();
    assert_lt!(tmp, time_now);
    assert_eq!(info.avatar_key.unwrap(), "");
    let session_id_get: SessionID = info.session_id.unwrap().into();
    assert_eq!(session_id_get, session.session_id);
    let members: HashSet<ID> = info.members.into_iter().map(|x| x.into()).collect();
    assert_eq!(
        members,
        HashSet::from_iter([a.lock().await.id, b.lock().await.id, c.lock().await.id,].into_iter())
    );
    let roles: HashSet<(ID, RoleId)> = info
        .roles
        .into_iter()
        .map(|x| (x.user_id.into(), RoleId(x.role as u64)))
        .collect();
    assert_eq!(
        roles,
        HashSet::from_iter([
            (a.lock().await.id, PredefinedRoles::Owner.into()),
            (b.lock().await.id, PredefinedRoles::Member.into()),
            (c.lock().await.id, PredefinedRoles::Member.into())
        ])
    );
    // permission denied
    let user_not_in_session = app.new_user().await.unwrap();
    assert_ok!(
        user_not_in_session
            .lock()
            .await
            .oc()
            .get_session_info(GetSessionInfoRequest {
                session_id: session.session_id.into(),
                query_values: vec![
                    QueryValues::Size.into(),
                    QueryValues::Name.into(),
                    QueryValues::Unspecified.into(),
                    QueryValues::AvatarKey.into(),
                    QueryValues::CreatedTime.into(),
                    QueryValues::SessionId.into(),
                ],
            })
            .await
    );
    // Cannot get member list
    let err = user_not_in_session
        .lock()
        .await
        .oc()
        .get_session_info(GetSessionInfoRequest {
            session_id: session.session_id.into(),
            query_values: vec![QueryValues::Members.into()],
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
    assert_eq!(err.message(), error_msg::PERMISSION_DENIED);

    // Cannot get roles list
    let err = user_not_in_session
        .lock()
        .await
        .oc()
        .get_session_info(GetSessionInfoRequest {
            session_id: session.session_id.into(),
            query_values: vec![QueryValues::Roles.into()],
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
    assert_eq!(err.message(), error_msg::PERMISSION_DENIED);
    app.async_drop().await;
}

#[tokio::test]
async fn set_session_info() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app
        .new_session_db_level(3, "session1", false)
        .await
        .unwrap();
    let (a, b, _c) = (
        session_user[0].clone(),
        session_user[1].clone(),
        session_user[2].clone(),
    );
    let get_timestamp = async || -> TimeStampUtc {
        a.lock()
            .await
            .oc()
            .get_session_info(GetSessionInfoRequest {
                session_id: session.session_id.into(),
                query_values: vec![QueryValues::UpdatedTime.into()],
            })
            .await
            .unwrap()
            .into_inner()
            .updated_time
            .unwrap()
            .try_into()
            .unwrap()
    };
    let original_timestamp = get_timestamp().await;
    let request = SetSessionInfoRequest {
        session_id: session.session_id.into(),
        name: Some("test name".to_owned()),
        description: Some("test description".to_owned()),
        avatar_key: Some("pic key".to_owned()),
    };
    a.lock()
        .await
        .oc()
        .set_session_info(request.clone())
        .await
        .unwrap();
    // check if the info was set
    let info = a
        .lock()
        .await
        .oc()
        .get_session_info(GetSessionInfoRequest {
            session_id: session.session_id.into(),
            query_values: vec![
                QueryValues::Name.into(),
                QueryValues::AvatarKey.into(),
                QueryValues::Description.into(),
            ],
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(info.name.unwrap(), "test name");
    assert_eq!(info.avatar_key.unwrap(), "pic key");
    assert_eq!(info.description.unwrap(), "test description");
    // check if timestamp was updated
    let now_timestamp = get_timestamp().await;
    assert_lt!(original_timestamp, now_timestamp);
    // without permission
    let err = b
        .lock()
        .await
        .oc()
        .set_session_info(request)
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
    assert_eq!(err.message(), error_msg::CANNOT_SET_NAME);
    app.async_drop().await;
}
