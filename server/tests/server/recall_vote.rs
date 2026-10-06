use base::constants::SessionID;
use client::TestApp;
use client::oc_helper::user::TestUserShared;
use pb::service::ourchat::msg_delivery::recall::v1::RecallMsgRequest;
use pb::service::ourchat::msg_delivery::recall_vote::v1::{
    GetRecallVoteRequest, StartRecallVoteRequest, VoteRecallRequest,
};
use pb::service::ourchat::msg_delivery::v1::{
    FetchSessionHistoryRequest, fetch_msgs_response::RespondEventType,
};

/// Whether the given message id still appears in the session history.
async fn history_contains(user: &TestUserShared, session_id: SessionID, msg_id: u64) -> bool {
    let before_time: pb::google::protobuf::Timestamp =
        user.lock().await.get_timestamp().await.into();
    let response = user
        .lock()
        .await
        .oc()
        .fetch_session_history(FetchSessionHistoryRequest {
            session_id: session_id.into(),
            before_time: Some(before_time),
            limit: 1000,
        })
        .await
        .unwrap()
        .into_inner();
    response.messages.iter().any(|m| m.msg_id == msg_id)
}

/// Move the deadline of a vote into the past, to simulate an expired vote
async fn expire_vote(app: &TestApp, vote_id: u64) {
    use sea_orm::{ConnectionTrait, Statement};
    let past = chrono::Utc::now() - chrono::Duration::hours(1);
    let values: [sea_orm::Value; 2] = [past.into(), (vote_id as i64).into()];
    let ret = app
        .get_db_connection()
        .execute_raw(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            "UPDATE recall_votes SET deadline = $1 WHERE id = $2",
            values,
        ))
        .await
        .unwrap();
    assert_eq!(ret.rows_affected(), 1, "expire_vote matched no vote");
}

/// B and C both approve -> the message is recalled
#[tokio::test]
async fn test_recall_vote_pass() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app
        .new_session_db_level(3, "session1", false)
        .await
        .unwrap();
    // session_user[0] is the Owner (C), the two members are A (sender) and
    // B (vote initiator)
    let (c, a, b) = (
        session_user[0].clone(),
        session_user[1].clone(),
        session_user[2].clone(),
    );
    let ret = a
        .lock()
        .await
        .send_msg(session.session_id, "hello", vec![], false)
        .await
        .unwrap();
    let msg_id = ret.into_inner().msg_id;

    let vote_id = b
        .lock()
        .await
        .oc()
        .start_recall_vote(StartRecallVoteRequest {
            msg_id,
            session_id: session.session_id.into(),
        })
        .await
        .unwrap()
        .into_inner()
        .vote_id;

    // B approves first (yes=1, not enough: 1*2 > 2 is false)
    b.lock()
        .await
        .oc()
        .vote_recall(VoteRecallRequest {
            vote_id,
            approve: true,
        })
        .await
        .unwrap();
    // C approves -> yes=2 -> 2*2 > 2 -> the vote passes and the message is
    // recalled
    c.lock()
        .await
        .oc()
        .vote_recall(VoteRecallRequest {
            vote_id,
            approve: true,
        })
        .await
        .unwrap();

    // the vote is settled as passed
    let vote = a
        .lock()
        .await
        .oc()
        .get_recall_vote(GetRecallVoteRequest { vote_id })
        .await
        .unwrap()
        .into_inner();
    assert!(vote.settled);
    assert!(vote.passed);
    assert_eq!(vote.yes_count, 2);
    assert_eq!(vote.no_count, 0);
    assert_eq!(vote.eligible_count, 2, "owner + B, the sender is excluded");
    assert_eq!(vote.msg_id, msg_id);
    assert_eq!(vote.initiator_id, u64::from(b.lock().await.id));
    assert_eq!(vote.session_id, u64::from(session.session_id));

    // B receives: the vote creation, B's vote, C's (settling) vote and the
    // recall event. The original message is gone: it was deleted from the
    // database by the recall and was published before B's live consumer
    // existed, so it is in neither the replay nor the live queue.
    let events = b.lock().await.fetch_msgs().fetch(4).await.unwrap();
    let recall_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.respond_event_type, Some(RespondEventType::Recall(_))))
        .collect();
    assert_eq!(recall_events.len(), 1, "{events:?}");
    match recall_events[0]
        .clone()
        .respond_event_type
        .expect("respond event set")
    {
        RespondEventType::Recall(recall) => assert_eq!(recall.msg_id, msg_id),
        other => panic!("unexpected event: {other:?}"),
    }
    // the settled vote notification has been broadcast as well
    let settled_notifications: Vec<_> = events
        .iter()
        .filter(|e| {
            matches!(
                &e.respond_event_type,
                Some(RespondEventType::RecallVoteNotification(n)) if n.settled && n.passed
            )
        })
        .collect();
    assert_eq!(settled_notifications.len(), 1, "{events:?}");

    // the original message is gone from the session history
    assert!(
        !history_contains(&a, session.session_id, msg_id).await,
        "recalled message should be gone"
    );

    // voting on a settled vote is rejected
    let e = a
        .lock()
        .await
        .oc()
        .vote_recall(VoteRecallRequest {
            vote_id,
            approve: true,
        })
        .await
        .unwrap_err();
    assert_eq!(e.code(), tonic::Code::FailedPrecondition, "{e:?}");

    app.async_drop().await;
}

/// The vote cannot reach a majority anymore -> settled as failed, message kept
#[tokio::test]
async fn test_recall_vote_fail() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app
        .new_session_db_level(3, "session1", false)
        .await
        .unwrap();
    let (c, a, b) = (
        session_user[0].clone(),
        session_user[1].clone(),
        session_user[2].clone(),
    );
    let msg_id = a
        .lock()
        .await
        .send_msg(session.session_id, "hello", vec![], false)
        .await
        .unwrap()
        .into_inner()
        .msg_id;

    let vote_id = b
        .lock()
        .await
        .oc()
        .start_recall_vote(StartRecallVoteRequest {
            msg_id,
            session_id: session.session_id.into(),
        })
        .await
        .unwrap()
        .into_inner()
        .vote_id;

    // B approves, C rejects: yes=1, no=1, no*2 >= eligible(2) -> failed
    b.lock()
        .await
        .oc()
        .vote_recall(VoteRecallRequest {
            vote_id,
            approve: true,
        })
        .await
        .unwrap();
    c.lock()
        .await
        .oc()
        .vote_recall(VoteRecallRequest {
            vote_id,
            approve: false,
        })
        .await
        .unwrap();

    let vote = b
        .lock()
        .await
        .oc()
        .get_recall_vote(GetRecallVoteRequest { vote_id })
        .await
        .unwrap()
        .into_inner();
    assert!(vote.settled);
    assert!(!vote.passed);
    assert_eq!(vote.yes_count, 1);
    assert_eq!(vote.no_count, 1);

    // the message is NOT recalled
    assert!(
        history_contains(&a, session.session_id, msg_id).await,
        "message should still exist"
    );
    // and no recall event was broadcast
    let events = b.lock().await.fetch_msgs().fetch(4).await.unwrap();
    assert!(
        !events
            .iter()
            .any(|e| matches!(e.respond_event_type, Some(RespondEventType::Recall(_)))),
        "{events:?}"
    );

    // a settled vote accepts no more votes
    let e = b
        .lock()
        .await
        .oc()
        .vote_recall(VoteRecallRequest {
            vote_id,
            approve: true,
        })
        .await
        .unwrap_err();
    assert_eq!(e.code(), tonic::Code::FailedPrecondition, "{e:?}");

    app.async_drop().await;
}

/// One user, one vote
#[tokio::test]
async fn test_recall_vote_duplicate_vote() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app
        .new_session_db_level(3, "session1", false)
        .await
        .unwrap();
    let (_c, a, b) = (
        session_user[0].clone(),
        session_user[1].clone(),
        session_user[2].clone(),
    );
    let msg_id = a
        .lock()
        .await
        .send_msg(session.session_id, "hello", vec![], false)
        .await
        .unwrap()
        .into_inner()
        .msg_id;
    let vote_id = b
        .lock()
        .await
        .oc()
        .start_recall_vote(StartRecallVoteRequest {
            msg_id,
            session_id: session.session_id.into(),
        })
        .await
        .unwrap()
        .into_inner()
        .vote_id;

    // B votes no. NOTE: with eligible=2 a single "no" already dooms the vote
    // (no*2 >= eligible), so the duplicate check must be tested with "yes"
    b.lock()
        .await
        .oc()
        .vote_recall(VoteRecallRequest {
            vote_id,
            approve: true,
        })
        .await
        .unwrap();
    let e = b
        .lock()
        .await
        .oc()
        .vote_recall(VoteRecallRequest {
            vote_id,
            approve: true,
        })
        .await
        .unwrap_err();
    assert_eq!(e.code(), tonic::Code::AlreadyExists, "{e:?}");

    app.async_drop().await;
}

/// Only session members may start or vote; the sender and permission holders
/// cannot start a vote
#[tokio::test]
async fn test_recall_vote_permission() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app
        .new_session_db_level(3, "session1", false)
        .await
        .unwrap();
    let (c, a, b) = (
        session_user[0].clone(),
        session_user[1].clone(),
        session_user[2].clone(),
    );
    let msg_id = a
        .lock()
        .await
        .send_msg(session.session_id, "hello", vec![], false)
        .await
        .unwrap()
        .into_inner()
        .msg_id;

    // a non-member cannot start a vote
    let outsider = app.new_user().await.unwrap();
    let e = outsider
        .lock()
        .await
        .oc()
        .start_recall_vote(StartRecallVoteRequest {
            msg_id,
            session_id: session.session_id.into(),
        })
        .await
        .unwrap_err();
    assert_eq!(e.code(), tonic::Code::PermissionDenied, "{e:?}");

    // the sender cannot start a vote on their own message
    let e = a
        .lock()
        .await
        .oc()
        .start_recall_vote(StartRecallVoteRequest {
            msg_id,
            session_id: session.session_id.into(),
        })
        .await
        .unwrap_err();
    assert_eq!(e.code(), tonic::Code::InvalidArgument, "{e:?}");

    // the Owner holds the RecallMsg permission and should recall directly
    let e = c
        .lock()
        .await
        .oc()
        .start_recall_vote(StartRecallVoteRequest {
            msg_id,
            session_id: session.session_id.into(),
        })
        .await
        .unwrap_err();
    assert_eq!(e.code(), tonic::Code::InvalidArgument, "{e:?}");

    let vote_id = b
        .lock()
        .await
        .oc()
        .start_recall_vote(StartRecallVoteRequest {
            msg_id,
            session_id: session.session_id.into(),
        })
        .await
        .unwrap()
        .into_inner()
        .vote_id;

    // a non-member cannot vote
    let e = outsider
        .lock()
        .await
        .oc()
        .vote_recall(VoteRecallRequest {
            vote_id,
            approve: true,
        })
        .await
        .unwrap_err();
    assert_eq!(e.code(), tonic::Code::PermissionDenied, "{e:?}");
    // the sender of the message cannot vote
    let e = a
        .lock()
        .await
        .oc()
        .vote_recall(VoteRecallRequest {
            vote_id,
            approve: true,
        })
        .await
        .unwrap_err();
    assert_eq!(e.code(), tonic::Code::PermissionDenied, "{e:?}");
    // a non-member cannot even query the vote
    let e = outsider
        .lock()
        .await
        .oc()
        .get_recall_vote(GetRecallVoteRequest { vote_id })
        .await
        .unwrap_err();
    assert_eq!(e.code(), tonic::Code::PermissionDenied, "{e:?}");

    app.async_drop().await;
}

/// Only one unsettled vote per message, and no votes on recalled messages
#[tokio::test]
async fn test_recall_vote_duplicate_start_and_recalled_msg() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app
        .new_session_db_level(3, "session1", false)
        .await
        .unwrap();
    let (c, a, b) = (
        session_user[0].clone(),
        session_user[1].clone(),
        session_user[2].clone(),
    );
    let msg_id = a
        .lock()
        .await
        .send_msg(session.session_id, "hello", vec![], false)
        .await
        .unwrap()
        .into_inner()
        .msg_id;
    let _vote_id = b
        .lock()
        .await
        .oc()
        .start_recall_vote(StartRecallVoteRequest {
            msg_id,
            session_id: session.session_id.into(),
        })
        .await
        .unwrap()
        .into_inner()
        .vote_id;

    // no second open vote for the same message
    let e = b
        .lock()
        .await
        .oc()
        .start_recall_vote(StartRecallVoteRequest {
            msg_id,
            session_id: session.session_id.into(),
        })
        .await
        .unwrap_err();
    assert_eq!(e.code(), tonic::Code::AlreadyExists, "{e:?}");

    // the Owner recalls the message directly (the vote is now pointless)
    c.lock()
        .await
        .oc()
        .recall_msg(RecallMsgRequest {
            msg_id,
            session_id: session.session_id.into(),
        })
        .await
        .unwrap();

    // a vote cannot be started for an already recalled message
    let e = b
        .lock()
        .await
        .oc()
        .start_recall_vote(StartRecallVoteRequest {
            msg_id,
            session_id: session.session_id.into(),
        })
        .await
        .unwrap_err();
    assert_eq!(e.code(), tonic::Code::NotFound, "{e:?}");

    app.async_drop().await;
}

/// Votes which reached no decision are settled as failed once expired
#[tokio::test]
async fn test_recall_vote_expired_lazy_settlement() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app
        .new_session_db_level(3, "session1", false)
        .await
        .unwrap();
    let (_c, a, b) = (
        session_user[0].clone(),
        session_user[1].clone(),
        session_user[2].clone(),
    );
    let msg_id = a
        .lock()
        .await
        .send_msg(session.session_id, "hello", vec![], false)
        .await
        .unwrap()
        .into_inner()
        .msg_id;
    let vote_id = b
        .lock()
        .await
        .oc()
        .start_recall_vote(StartRecallVoteRequest {
            msg_id,
            session_id: session.session_id.into(),
        })
        .await
        .unwrap()
        .into_inner()
        .vote_id;

    // simulate an expired vote
    expire_vote(&app, vote_id).await;

    // voting is rejected and settles the vote lazily
    let e = b
        .lock()
        .await
        .oc()
        .vote_recall(VoteRecallRequest {
            vote_id,
            approve: true,
        })
        .await
        .unwrap_err();
    assert_eq!(e.code(), tonic::Code::FailedPrecondition, "{e:?}");

    // the vote is settled as failed ...
    let vote = b
        .lock()
        .await
        .oc()
        .get_recall_vote(GetRecallVoteRequest { vote_id })
        .await
        .unwrap()
        .into_inner();
    assert!(vote.settled);
    assert!(!vote.passed);
    assert_eq!(vote.yes_count, 0);

    // ... the message is kept ...
    assert!(
        history_contains(&a, session.session_id, msg_id).await,
        "expired vote must not recall the message"
    );
    // ... and no further votes are accepted
    let e = b
        .lock()
        .await
        .oc()
        .vote_recall(VoteRecallRequest {
            vote_id,
            approve: true,
        })
        .await
        .unwrap_err();
    assert_eq!(e.code(), tonic::Code::FailedPrecondition, "{e:?}");

    // once settled, a new vote may be started for the same message
    let vote_id2 = b
        .lock()
        .await
        .oc()
        .start_recall_vote(StartRecallVoteRequest {
            msg_id,
            session_id: session.session_id.into(),
        })
        .await
        .unwrap()
        .into_inner()
        .vote_id;
    assert_ne!(vote_id, vote_id2);

    app.async_drop().await;
}
