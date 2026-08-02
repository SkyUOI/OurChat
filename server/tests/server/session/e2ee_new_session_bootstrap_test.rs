//! Tests that creating an E2EE session via `NewSession` immediately bootstraps
//! room-key distribution (the E2EE completion fix). Before this fix, a freshly
//! created E2EE session never distributed keys until an explicit
//! `E2eeizeSession` call or room-key expiry, so encryption never actually
//! started. Now the creator receives `UpdateRoomKey` (to generate a key) plus
//! one `SendRoomKey` per other member (to wrap it).

use bytes::Bytes;
use client::TestApp;
use pb::service::ourchat::{
    msg_delivery::v1::fetch_msgs_response::RespondEventType,
    session::new_session::v1::NewSessionRequest,
};
use rsa::{RsaPublicKey, pkcs1::DecodeRsaPublicKey as _};

#[tokio::test]
async fn new_e2ee_session_bootstraps_room_key_distribution() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    let user_a = app.new_user().await.unwrap();
    let user_b = app.new_user().await.unwrap();
    let (aid, bid) = (user_a.lock().await.id, user_b.lock().await.id);

    // Make them friends so user_b auto-joins the session (no verification).
    app.create_friendship(aid, bid).await.unwrap();

    // user_a creates an E2EE session that includes user_b.
    let resp = user_a
        .lock()
        .await
        .oc()
        .new_session(NewSessionRequest {
            members: vec![bid.into()],
            e2ee_on: true,
            ..Default::default()
        })
        .await
        .expect("create E2EE session");
    let session_id: base::constants::SessionID = resp.into_inner().session_id.into();

    // The creator must receive: 1x UpdateRoomKey + 1x SendRoomKey (for user_b).
    let msgs = user_a.lock().await.fetch_msgs().fetch(2).await.unwrap();
    assert_eq!(
        msgs.len(),
        2,
        "creator should receive update + send room key"
    );

    let RespondEventType::UpdateRoomKey(update) = msgs[0].respond_event_type.clone().unwrap()
    else {
        panic!(
            "first message must be UpdateRoomKey, got {:?}",
            msgs[0].respond_event_type
        );
    };
    assert_eq!(update.session_id, *session_id);

    let RespondEventType::SendRoomKey(send) = msgs[1].respond_event_type.clone().unwrap() else {
        panic!(
            "second message must be SendRoomKey, got {:?}",
            msgs[1].respond_event_type
        );
    };
    assert_eq!(send.session_id, *session_id);
    assert_eq!(
        send.sender, *bid,
        "SendRoomKey must carry the other member's id"
    );
    // The embedded public key must be user_b's registered key, and valid DER.
    assert_eq!(send.public_key, user_b.lock().await.public_key_bytes());
    RsaPublicKey::from_pkcs1_der(&send.public_key)
        .expect("embedded public key must be valid PKCS#1 DER");

    app.async_drop().await;
}

#[tokio::test]
async fn new_plaintext_session_does_not_distribute_room_key() {
    // Regression guard: a non-E2EE session must NOT emit any room-key messages.
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    let user_a = app.new_user().await.unwrap();
    let user_b = app.new_user().await.unwrap();
    let (aid, bid) = (user_a.lock().await.id, user_b.lock().await.id);
    app.create_friendship(aid, bid).await.unwrap();

    let _resp = user_a
        .lock()
        .await
        .oc()
        .new_session(NewSessionRequest {
            members: vec![bid.into()],
            e2ee_on: false,
            ..Default::default()
        })
        .await
        .unwrap();

    // Wait briefly, then ensure no room-key messages arrive (timeout expected).
    let result = user_a
        .lock()
        .await
        .fetch_msgs()
        .set_timeout(std::time::Duration::from_millis(800))
        .fetch(1)
        .await;
    // Either times out, or delivers zero room-key notifications.
    match result {
        // A timeout is the expected outcome (no messages emitted at all).
        Err(_) => {}
        Ok(msgs) => {
            for m in &msgs {
                if let Some(ev) = m.respond_event_type.clone() {
                    use pb::service::ourchat::msg_delivery::v1::fetch_msgs_response::RespondEventType as E;
                    assert!(
                        !matches!(
                            ev,
                            E::UpdateRoomKey(_) | E::SendRoomKey(_) | E::ReceiveRoomKey(_)
                        ),
                        "plaintext session must not emit room-key messages: {ev:?}"
                    );
                }
            }
        }
    }

    // Touch the import so it is not flagged unused when the Ok branch is taken.
    let _: Option<Bytes> = None;

    app.async_drop().await;
}
