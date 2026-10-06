use core::panic;

use bytes::Bytes;
use client::{TestApp, oc_helper::TestSession, oc_helper::user::TestUserShared};
use migration::predefined::SessionInvitationPolicy;
use pb::service::ourchat::{
    msg_delivery::v1::fetch_msgs_response::RespondEventType,
    session::{
        accept_join_session_invitation::v1::AcceptJoinSessionInvitationRequest,
        invite_user_to_session::v1::InviteUserToSessionRequest,
        session_room_key::v1::SendRoomKeyRequest,
    },
    set_account_info::v1::SetSelfInfoRequest,
};
use rsa::{RsaPublicKey, pkcs1::DecodeRsaPublicKey as _};
use sea_orm::TransactionTrait;
use server::db::session::in_session;
use server::process::error_msg::{SESSION_INVITATION_FRIENDS_ONLY, SESSION_INVITATION_NOT_ALLOWED};

#[tokio::test]
async fn invite_user_to_session_success() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app.new_session_db_level(2, "session1", true).await.unwrap();
    let a = session_user[0].clone();
    let b = session_user[1].clone();
    let c = app.new_user().await.unwrap();
    let (aid, _bid, cid) = (a.lock().await.id, b.lock().await.id, c.lock().await.id);

    a.lock()
        .await
        .oc()
        .invite_user_to_session(InviteUserToSessionRequest {
            session_id: session.session_id.into(),
            invitee: cid.into(),
            leave_message: Some("hi".to_owned()),
        })
        .await
        .unwrap();
    let invite_request = c.lock().await.fetch_msgs().fetch(1).await.unwrap();
    assert_eq!(invite_request.len(), 1);
    let RespondEventType::InviteUserToSession(invite_request) = invite_request
        .into_iter()
        .next()
        .unwrap()
        .respond_event_type
        .unwrap()
    else {
        panic!("invite request is not InviteSession",);
    };
    assert_eq!(invite_request.session_id, *session.session_id);
    assert_eq!(invite_request.inviter_id, *aid);
    assert_eq!(invite_request.leave_message, Some("hi".to_string()));
    assert!(
        !in_session(cid, session.session_id, app.get_db_connection())
            .await
            .unwrap()
    );
    c.lock()
        .await
        .oc()
        .accept_join_session_invitation(AcceptJoinSessionInvitationRequest {
            session_id: session.session_id.into(),
            accepted: true,
            inviter_id: aid.into(),
        })
        .await
        .unwrap();
    let accept_approval = a.lock().await.fetch_msgs().fetch(2).await.unwrap();
    assert_eq!(accept_approval.len(), 2);
    let RespondEventType::AcceptSessionApproval(accept_approval) = accept_approval
        .into_iter()
        .nth(1)
        .unwrap()
        .respond_event_type
        .unwrap()
    else {
        panic!("accept notification is not AcceptSessionApproval",);
    };
    assert!(
        in_session(cid, session.session_id, app.get_db_connection())
            .await
            .unwrap()
    );
    assert_eq!(accept_approval.session_id, *session.session_id);
    assert_eq!(accept_approval.invitee_id, *cid);
    assert_eq!(
        accept_approval.public_key,
        Some(c.lock().await.public_key_bytes())
    );
    let public_key = RsaPublicKey::from_pkcs1_der(&c.lock().await.public_key_bytes()).unwrap();
    let room_key = TestSession::generate_room_key();
    let mut rng = rand::rng();
    let encrypted_room_key: Bytes = public_key
        .encrypt(&mut rng, utils::oaep_padding(), &room_key)
        .unwrap()
        .into();
    a.lock()
        .await
        .oc()
        .send_room_key(SendRoomKeyRequest {
            session_id: session.session_id.into(),
            user_id: cid.into(),
            room_key: encrypted_room_key,
        })
        .await
        .unwrap();
    let room_key_notification = c.lock().await.fetch_msgs().fetch(2).await.unwrap();
    assert_eq!(room_key_notification.len(), 2);
    let RespondEventType::ReceiveRoomKey(room_key_notification) = room_key_notification
        .into_iter()
        .nth(1)
        .unwrap()
        .respond_event_type
        .unwrap()
    else {
        panic!("room key request is not ReceiveRoomKey");
    };
    assert_eq!(room_key_notification.session_id, *session.session_id);
    assert_eq!(room_key_notification.user_id, *aid);
    let received_encrypted_room_key = room_key_notification.room_key;
    let received_room_key: Bytes = c
        .lock()
        .await
        .key_pair
        .0
        .decrypt(utils::oaep_padding(), &received_encrypted_room_key)
        .unwrap()
        .into();
    assert_eq!(received_room_key, room_key);
    app.async_drop().await
}

#[tokio::test]
async fn invite_user_to_session_reject() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app.new_session_db_level(2, "session1", true).await.unwrap();
    let a = session_user[0].clone();
    let b = session_user[1].clone();
    let c = app.new_user().await.unwrap();
    let (aid, _bid, cid) = (a.lock().await.id, b.lock().await.id, c.lock().await.id);

    a.lock()
        .await
        .oc()
        .invite_user_to_session(InviteUserToSessionRequest {
            session_id: session.session_id.into(),
            invitee: cid.into(),
            leave_message: Some("hi".to_owned()),
        })
        .await
        .unwrap();
    let invite_request = c.lock().await.fetch_msgs().fetch(1).await.unwrap();
    assert_eq!(invite_request.len(), 1);
    let RespondEventType::InviteUserToSession(invite_request) = invite_request
        .into_iter()
        .next()
        .unwrap()
        .respond_event_type
        .unwrap()
    else {
        panic!("invite request is not InviteSession",);
    };
    assert_eq!(invite_request.session_id, *session.session_id);
    assert_eq!(invite_request.inviter_id, *aid);
    assert_eq!(invite_request.leave_message, Some("hi".to_string()));
    assert!(
        !in_session(cid, session.session_id, app.get_db_connection())
            .await
            .unwrap()
    );
    c.lock()
        .await
        .oc()
        .accept_join_session_invitation(AcceptJoinSessionInvitationRequest {
            session_id: session.session_id.into(),
            accepted: false,
            inviter_id: aid.into(),
        })
        .await
        .unwrap();
    let accept_approval = a.lock().await.fetch_msgs().fetch(2).await.unwrap();
    assert_eq!(accept_approval.len(), 2);
    let RespondEventType::AcceptSessionApproval(accept_approval) = accept_approval
        .into_iter()
        .nth(1)
        .unwrap()
        .respond_event_type
        .unwrap()
    else {
        panic!("accept notification is not AcceptJoinInSession");
    };
    assert!(
        !in_session(cid, session.session_id, app.get_db_connection())
            .await
            .unwrap()
    );
    assert_eq!(accept_approval.session_id, *session.session_id);
    assert_eq!(accept_approval.invitee_id, *cid);
    assert_eq!(accept_approval.public_key, None);
    app.async_drop().await
}

// ── session invitation policy tests ──

/// Set the session invitation policy of a user through the RPC.
async fn set_session_invitation_policy(user: &TestUserShared, policy: i32) {
    user.lock()
        .await
        .oc()
        .set_self_info(SetSelfInfoRequest {
            session_invitation_policy: Some(policy),
            ..Default::default()
        })
        .await
        .unwrap();
}

/// An ALLOW_ALL invitee (the default) can be invited by anyone.
#[tokio::test]
async fn invite_user_to_session_allow_all_policy() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app.new_session_db_level(2, "session1", true).await.unwrap();
    let a = session_user[0].clone();
    let c = app.new_user().await.unwrap();
    let cid = c.lock().await.id;

    // explicitly set the (default) policy
    set_session_invitation_policy(&c, SessionInvitationPolicy::AllowAll as i32).await;

    // a stranger can invite
    a.lock()
        .await
        .oc()
        .invite_user_to_session(InviteUserToSessionRequest {
            session_id: session.session_id.into(),
            invitee: cid.into(),
            leave_message: None,
        })
        .await
        .unwrap();
    let invite_request = c.lock().await.fetch_msgs().fetch(1).await.unwrap();
    assert_eq!(invite_request.len(), 1);
    let RespondEventType::InviteUserToSession(invite_request) = invite_request
        .into_iter()
        .next()
        .unwrap()
        .respond_event_type
        .unwrap()
    else {
        panic!("invite request is not InviteUserToSession");
    };
    assert_eq!(invite_request.session_id, *session.session_id);
    app.async_drop().await
}

/// A FRIENDS_ONLY invitee cannot be invited by a stranger, but a friend can
/// still invite them.
#[tokio::test]
async fn invite_user_to_session_friends_only_policy() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app.new_session_db_level(2, "session1", true).await.unwrap();
    let a = session_user[0].clone();
    let c = app.new_user().await.unwrap();
    let (aid, cid) = (a.lock().await.id, c.lock().await.id);

    // c only accepts invitations from friends
    set_session_invitation_policy(&c, SessionInvitationPolicy::FriendsOnly as i32).await;

    // a is not a friend of c -> rejected
    let err = a
        .lock()
        .await
        .oc()
        .invite_user_to_session(InviteUserToSessionRequest {
            session_id: session.session_id.into(),
            invitee: cid.into(),
            leave_message: None,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
    assert_eq!(err.message(), SESSION_INVITATION_FRIENDS_ONLY);

    // become friends
    let txn = app.get_db_connection().begin().await.unwrap();
    server::db::friend::add_friend(aid, cid, None, None, &txn)
        .await
        .unwrap();
    txn.commit().await.unwrap();

    // now the invitation goes through
    a.lock()
        .await
        .oc()
        .invite_user_to_session(InviteUserToSessionRequest {
            session_id: session.session_id.into(),
            invitee: cid.into(),
            leave_message: None,
        })
        .await
        .unwrap();
    let invite_request = c.lock().await.fetch_msgs().fetch(1).await.unwrap();
    assert_eq!(invite_request.len(), 1);
    let RespondEventType::InviteUserToSession(invite_request) = invite_request
        .into_iter()
        .next()
        .unwrap()
        .respond_event_type
        .unwrap()
    else {
        panic!("invite request is not InviteUserToSession");
    };
    assert_eq!(invite_request.session_id, *session.session_id);
    app.async_drop().await
}

/// A NOBODY invitee rejects every session invitation, even from friends.
#[tokio::test]
async fn invite_user_to_session_nobody_policy() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app.new_session_db_level(2, "session1", true).await.unwrap();
    let a = session_user[0].clone();
    let c = app.new_user().await.unwrap();
    let (aid, cid) = (a.lock().await.id, c.lock().await.id);

    // c rejects all invitations
    set_session_invitation_policy(&c, SessionInvitationPolicy::Nobody as i32).await;

    // a stranger is rejected
    let err = a
        .lock()
        .await
        .oc()
        .invite_user_to_session(InviteUserToSessionRequest {
            session_id: session.session_id.into(),
            invitee: cid.into(),
            leave_message: None,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
    assert_eq!(err.message(), SESSION_INVITATION_NOT_ALLOWED);

    // even a friend is rejected
    let txn = app.get_db_connection().begin().await.unwrap();
    server::db::friend::add_friend(aid, cid, None, None, &txn)
        .await
        .unwrap();
    txn.commit().await.unwrap();
    let err = a
        .lock()
        .await
        .oc()
        .invite_user_to_session(InviteUserToSessionRequest {
            session_id: session.session_id.into(),
            invitee: cid.into(),
            leave_message: None,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
    assert_eq!(err.message(), SESSION_INVITATION_NOT_ALLOWED);
    app.async_drop().await
}
