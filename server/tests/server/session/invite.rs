use core::panic;

use bytes::Bytes;
use client::{TestApp, oc_helper::TestSession};
use pb::service::ourchat::{
    msg_delivery::v1::fetch_msgs_response::RespondEventType,
    session::{
        accept_join_session_invitation::v1::AcceptJoinSessionInvitationRequest,
        invite_user_to_session::v1::InviteUserToSessionRequest,
        session_room_key::v1::SendRoomKeyRequest,
    },
};
use rand::rngs::OsRng;
use rsa::{Pkcs1v15Encrypt, RsaPublicKey, pkcs1::DecodeRsaPublicKey as _};
use server::db::session::in_session;

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
    let encrypted_room_key: Bytes = public_key
        .encrypt(&mut OsRng, Pkcs1v15Encrypt, &room_key)
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
        .decrypt(Pkcs1v15Encrypt, &received_encrypted_room_key)
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
