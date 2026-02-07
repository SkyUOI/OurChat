use std::time::Duration;

use bytes::Bytes;
use client::{TestApp, oc_helper::TestSession};
use pb::service::ourchat::{
    msg_delivery::v1::fetch_msgs_response::RespondEventType,
    session::{leave_session::v1::LeaveSessionRequest, session_room_key::v1::SendRoomKeyRequest},
};
use rsa::{RsaPublicKey, pkcs1::DecodeRsaPublicKey as _};
use server::db::session::get_session_by_id;
use tokio::time::sleep;

#[tokio::test]
async fn e2ee_update_timeout() {
    let (mut config, args) = TestApp::get_test_config().unwrap();
    config.main_cfg.room_key_duration = Duration::from_secs(1);
    let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args), |_| {})
        .await
        .unwrap();
    let (session_user, session) = app.new_session_db_level(2, "session1", true).await.unwrap();
    sleep(Duration::from_secs(1)).await;
    let a = session_user[0].clone();
    let b = session_user[1].clone();
    let (_aid, bid) = (a.lock().await.id, b.lock().await.id);
    a.lock()
        .await
        .send_msg(session.session_id, "encrypted Hi", vec![], true)
        .await
        .unwrap();
    let msgs = a.lock().await.fetch_msgs().fetch(3).await.unwrap();
    assert_eq!(msgs.len(), 3);
    let RespondEventType::UpdateRoomKey(update_room_key) =
        msgs[1].respond_event_type.clone().unwrap()
    else {
        panic!("what is accepted is not update room key");
    };
    let RespondEventType::SendRoomKey(send_room_key) = msgs[2].respond_event_type.clone().unwrap()
    else {
        panic!("what is accepted is not send room key")
    };
    assert_eq!(update_room_key.session_id, *session.session_id);
    assert_eq!(send_room_key.session_id, *session.session_id);
    assert_eq!(send_room_key.sender, *bid);
    assert_eq!(send_room_key.public_key, b.lock().await.public_key_bytes());
    let public_key = RsaPublicKey::from_pkcs1_der(&send_room_key.public_key).unwrap();
    let new_room_key = TestSession::generate_room_key();
    let mut rng = rand::rng();
    let encrypted_room_key: Bytes = public_key
        .encrypt(&mut rng, utils::oaep_padding(), &new_room_key)
        .unwrap()
        .into();
    a.lock()
        .await
        .oc()
        .send_room_key(SendRoomKeyRequest {
            session_id: session.session_id.into(),
            user_id: bid.into(),
            room_key: encrypted_room_key,
        })
        .await
        .unwrap();
    let msgs = b.lock().await.fetch_msgs().fetch(4).await.unwrap();
    assert_eq!(msgs.len(), 4);
    let RespondEventType::Msg(_) = msgs[0].respond_event_type.clone().unwrap() else {
        panic!("what is accepted is not msg")
    };
    let RespondEventType::ReceiveRoomKey(receive_room_key) =
        msgs[3].respond_event_type.clone().unwrap()
    else {
        panic!("what is accepted is not receive room key");
    };
    assert_eq!(receive_room_key.session_id, *session.session_id);
    let received_encrypted_room_key = receive_room_key.room_key;
    let received_room_key: Bytes = b
        .lock()
        .await
        .key_pair
        .0
        .decrypt(utils::oaep_padding(), &received_encrypted_room_key)
        .unwrap()
        .into();
    assert_eq!(received_room_key, new_room_key);
    app.async_drop().await
}

#[tokio::test]
async fn e2ee_update_member_leaving() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app.new_session_db_level(3, "session1", true).await.unwrap();
    let a = session_user[0].clone();
    let b = session_user[1].clone();
    let c = session_user[2].clone();
    let (_aid, bid, _cid) = (a.lock().await.id, b.lock().await.id, c.lock().await.id);
    c.lock()
        .await
        .oc()
        .leave_session(LeaveSessionRequest {
            session_id: session.session_id.into(),
        })
        .await
        .unwrap();
    let session_model = get_session_by_id(session.session_id, app.get_db_connection())
        .await
        .unwrap()
        .unwrap();
    assert!(session_model.leaving_to_process);
    a.lock()
        .await
        .send_msg(session.session_id, "encrypted Hi", vec![], true)
        .await
        .unwrap();
    let msgs = a.lock().await.fetch_msgs().fetch(3).await.unwrap();
    assert_eq!(msgs.len(), 3);
    let RespondEventType::UpdateRoomKey(update_room_key) =
        msgs[1].respond_event_type.clone().unwrap()
    else {
        panic!("what is accepted is not update room key");
    };
    let RespondEventType::SendRoomKey(send_room_key) = msgs[2].respond_event_type.clone().unwrap()
    else {
        panic!("what is accepted is not send room key")
    };
    assert_eq!(update_room_key.session_id, *session.session_id);
    assert_eq!(send_room_key.session_id, *session.session_id);
    assert_eq!(send_room_key.sender, *bid);
    assert_eq!(send_room_key.public_key, b.lock().await.public_key_bytes());
    let public_key = RsaPublicKey::from_pkcs1_der(&send_room_key.public_key).unwrap();
    let new_room_key = TestSession::generate_room_key();
    let mut rng = rand::rng();
    let encrypted_room_key: Bytes = public_key
        .encrypt(&mut rng, utils::oaep_padding(), &new_room_key)
        .unwrap()
        .into();
    a.lock()
        .await
        .oc()
        .send_room_key(SendRoomKeyRequest {
            session_id: session.session_id.into(),
            user_id: bid.into(),
            room_key: encrypted_room_key,
        })
        .await
        .unwrap();
    let msgs = b.lock().await.fetch_msgs().fetch(4).await.unwrap();
    assert_eq!(msgs.len(), 4);
    let RespondEventType::Msg(_) = msgs[0].respond_event_type.clone().unwrap() else {
        panic!("what is accepted is not msg")
    };
    let RespondEventType::ReceiveRoomKey(receive_room_key) =
        msgs[3].respond_event_type.clone().unwrap()
    else {
        panic!("what is accepted is not receive room key");
    };
    assert_eq!(receive_room_key.session_id, *session.session_id);
    let received_encrypted_room_key = receive_room_key.room_key;
    let received_room_key: Bytes = b
        .lock()
        .await
        .key_pair
        .0
        .decrypt(utils::oaep_padding(), &received_encrypted_room_key)
        .unwrap()
        .into();
    assert_eq!(received_room_key, new_room_key);
    app.async_drop().await
}
