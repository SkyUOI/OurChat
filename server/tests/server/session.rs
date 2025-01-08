mod role;

use std::collections::HashSet;

use base::consts::ID;
use base::time::from_google_timestamp;
use claims::assert_lt;
use client::TestApp;
use migration::m20241229_022701_add_role_for_session::PreDefinedRoles;
use pb::ourchat::session::{
    get_session_info::v1::{GetSessionInfoRequest, QueryValues},
    new_session::v1::NewSessionRequest,
};

#[tokio::test]
async fn session_create() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user1 = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();
    let user3 = app.new_user().await.unwrap();
    // try to create a session in two users
    let req = NewSessionRequest {
        members: vec![
            user2.lock().await.ocid.clone(),
            user3.lock().await.ocid.clone(),
        ],
        ..Default::default()
    };
    // get new session response
    let ret = user1.lock().await.oc().new_session(req).await.unwrap();
    let ret = ret.into_inner();
    let session_id = ret.session_id;
    // verify user2 received the invite
    // let resp = user2.lock().await.recv().await.unwrap();
    // let json: InviteSession = serde_json::from_str(resp.to_text().unwrap()).unwrap();
    // assert_eq!(json.inviter_id, user1.lock().await.ocid);
    // assert_eq!(json.code, MessageType::InviteSession);
    // assert!(json.message.is_empty());
    // assert_eq!(json.session_id, session_id);

    // verify user3 received the invite
    // user3.lock().await.ocid_login().await.unwrap();
    // let resp = user3.lock().await.recv().await.unwrap();
    // dbg!(&resp);
    // let json: InviteSession = serde_json::from_str(resp.to_text().unwrap()).unwrap();
    // assert_eq!(json.inviter_id, user1.lock().await.ocid);
    // assert_eq!(json.code, MessageType::InviteSession);
    // assert!(json.message.is_empty());
    // assert_eq!(json.session_id, session_id);
    app.async_drop().await;
}

#[tokio::test]
async fn get_session_info() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app.new_session_db_level(3, "session1").await.unwrap();
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
    assert_lt!(
        from_google_timestamp(&info.created_time.unwrap()).unwrap(),
        time_now
    );
    assert_eq!(info.avatar_key.unwrap(), "");
    let session_id_get: ID = info.session_id.unwrap().into();
    assert_eq!(session_id_get, session.session_id);
    let members: HashSet<ID> = info.members.into_iter().map(|x| x.into()).collect();
    assert_eq!(
        members,
        HashSet::from_iter([a.lock().await.id, b.lock().await.id, c.lock().await.id,].into_iter())
    );
    let roles: HashSet<(ID, u64)> = info
        .roles
        .into_iter()
        .map(|x| (x.user_id.into(), x.role))
        .collect();
    assert_eq!(
        roles,
        HashSet::from_iter([
            (a.lock().await.id, PreDefinedRoles::Member.into()),
            (b.lock().await.id, PreDefinedRoles::Member.into()),
            (c.lock().await.id, PreDefinedRoles::Member.into())
        ])
    );
    app.async_drop().await;
}

#[tokio::test]
async fn set_session_info() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app.new_session_db_level(3, "session1").await.unwrap();
    let (a, b, c) = (
        session_user[0].clone(),
        session_user[1].clone(),
        session_user[2].clone(),
    );
    app.async_drop().await;
}
