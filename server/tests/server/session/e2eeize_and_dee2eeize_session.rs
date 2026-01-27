use bytes::Bytes;
use claims::assert_err;
use client::{TestApp, oc_helper::TestSession};
use pb::service::ourchat::{
    msg_delivery::v1::fetch_msgs_response::RespondEventType,
    session::{
        e2eeize_and_dee2eeize_session::v1::{Dee2eeizeSessionRequest, E2eeizeSessionRequest},
        session_room_key::v1::SendRoomKeyRequest,
    },
};
use rsa::{RsaPublicKey, pkcs1::DecodeRsaPublicKey as _};
use server::db::session::get_session_by_id;

#[tokio::test]
pub async fn test_e2eeize_session() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app
        .new_session_db_level(2, "session1", false)
        .await
        .unwrap();
    let a = session_user[0].clone();
    let b = session_user[1].clone();
    let (_aid, bid) = (a.lock().await.id, b.lock().await.id);
    assert_err!(
        b.lock()
            .await
            .oc()
            .e2eeize_session(E2eeizeSessionRequest {
                session_id: session.session_id.into(),
            })
            .await
    );
    a.lock()
        .await
        .oc()
        .e2eeize_session(E2eeizeSessionRequest {
            session_id: session.session_id.into(),
        })
        .await
        .unwrap();

    let msgs = a.lock().await.fetch_msgs().fetch(2).await.unwrap();
    assert_eq!(msgs.len(), 2);
    let RespondEventType::UpdateRoomKey(update_room_key) =
        msgs[0].respond_event_type.clone().unwrap()
    else {
        panic!("what is accepted is not update room key");
    };
    let RespondEventType::SendRoomKey(send_room_key) = msgs[1].respond_event_type.clone().unwrap()
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
    let msgs = b.lock().await.fetch_msgs().fetch(3).await.unwrap();
    assert_eq!(msgs.len(), 3);
    let RespondEventType::ReceiveRoomKey(receive_room_key) =
        msgs[2].respond_event_type.clone().unwrap()
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
pub async fn test_dee2eeize_session() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app.new_session_db_level(2, "session1", true).await.unwrap();
    let a = session_user[0].clone();
    let b: std::sync::Arc<tokio::sync::Mutex<client::TestUser>> = session_user[1].clone();
    let (_aid, _bid) = (a.lock().await.id, b.lock().await.id);
    assert_err!(
        b.lock()
            .await
            .oc()
            .e2eeize_session(E2eeizeSessionRequest {
                session_id: session.session_id.into(),
            })
            .await
    );
    a.lock()
        .await
        .oc()
        .dee2eeize_session(Dee2eeizeSessionRequest {
            session_id: session.session_id.into(),
        })
        .await
        .unwrap();
    let session = get_session_by_id(session.session_id, app.get_db_connection())
        .await
        .unwrap()
        .unwrap();
    assert!(!session.e2ee_on);
    assert_err!(
        a.lock()
            .await
            .send_msg(session.session_id.into(), "encrypted Hi", vec![], true,)
            .await
    );
    app.async_drop().await
}
