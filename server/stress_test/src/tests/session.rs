use crate::UsersGroup;
use crate::framework::{Record, Report, StressTest, run_session_stress_test};
use base::constants::{ID, SessionID};
use dashmap::DashMap;
use migration::predefined::PredefinedRoles;
use pb::service::ourchat::session::accept_join_session_invitation::v1::AcceptJoinSessionInvitationRequest;
use pb::service::ourchat::session::add_role::v1::AddRoleRequest;
use pb::service::ourchat::session::allow_user_join_session::v1::AllowUserJoinSessionRequest;
use pb::service::ourchat::session::ban::v1::BanUserRequest;
use pb::service::ourchat::session::delete_session::v1::DeleteSessionRequest;
use pb::service::ourchat::session::e2eeize_and_dee2eeize_session::v1::{
    Dee2eeizeSessionRequest, E2eeizeSessionRequest,
};
use pb::service::ourchat::session::get_role::v1::GetRoleRequest;
use pb::service::ourchat::session::get_session_info::v1::{
    GetSessionInfoRequest, QueryValues as SessionQueryValues,
};
use pb::service::ourchat::session::invite_user_to_session::v1::InviteUserToSessionRequest;
use pb::service::ourchat::session::join_session::v1::JoinSessionRequest;
use pb::service::ourchat::session::kick::v1::KickUserRequest;
use pb::service::ourchat::session::leave_session::v1::LeaveSessionRequest;
use pb::service::ourchat::session::mute::v1::MuteUserRequest;
use pb::service::ourchat::session::new_session::v1::NewSessionRequest;
use pb::service::ourchat::session::session_room_key::v1::SendRoomKeyRequest;
use pb::service::ourchat::session::set_role::v1::SetRoleRequest;
use pb::service::ourchat::session::set_session_info::v1::SetSessionInfoRequest;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use derive::register_test;

#[register_test("New Session", WithSessions)]
pub async fn test_new_session(
    users: &UsersGroup,
    report: &mut Report,
) -> anyhow::Result<Arc<DashMap<ID, SessionID>>> {
    tracing::info!("▶️  Running test: 'new_session'");
    let mut stress_test = StressTest::builder().set_concurrency(100).set_requests(100);
    let users = users.clone();
    let idx = Arc::new(AtomicUsize::new(0));
    let sessions = Arc::new(DashMap::new());
    let sessions_ret = sessions.clone();
    let output = stress_test
        .stress_test(move || {
            let now = idx.fetch_add(1, Ordering::Relaxed);
            let user = users[now].clone();
            let sessions = sessions.clone();
            async move {
                // Get user_id in one lock, then make the call in another to avoid deadlock
                let user_id = {
                    let u = user.lock().await;
                    u.id
                };
                match user
                    .lock()
                    .await
                    .oc()
                    .new_session(NewSessionRequest {
                        members: vec![],
                        name: Some(format!("session_{}", rand::random::<u32>())),
                        leave_message: None,
                        avatar_key: None,
                        e2ee_on: false,
                    })
                    .await
                {
                    Ok(resp) => {
                        let session_id = SessionID(resp.into_inner().session_id);
                        sessions.insert(user_id, session_id);
                        true
                    }
                    Err(e) => {
                        tracing::error!("Failed to create session: {}", e);
                        false
                    }
                }
            }
        })
        .await;
    report.add_record(Record::new("new_session", output));
    Ok(sessions_ret)
}

#[register_test("Get Session Info", WithSessions)]
pub async fn test_get_session_info(
    sessions: Arc<DashMap<ID, SessionID>>,
    users: &UsersGroup,
    report: &mut Report,
) {
    run_session_stress_test(
        report,
        "get_session_info",
        sessions,
        users,
        100,
        100,
        |user, _now, _users, sessions| async move {
            // Get user_id and session_id in one lock to avoid deadlock
            let session_id = {
                let u = user.lock().await;
                sessions.get(&u.id).map(|s| *s)
            };
            if let Some(session_id) = session_id {
                user.lock()
                    .await
                    .oc()
                    .get_session_info(GetSessionInfoRequest {
                        session_id: session_id.0,
                        query_values: vec![
                            SessionQueryValues::SessionId.into(),
                            SessionQueryValues::Name.into(),
                            SessionQueryValues::Members.into(),
                            SessionQueryValues::CreatedTime.into(),
                            SessionQueryValues::Size.into(),
                        ],
                    })
                    .await
                    .is_ok()
            } else {
                false
            }
        },
    )
    .await;
}

#[register_test("Set Session Info", WithSessions)]
pub async fn test_set_session_info(
    sessions: Arc<DashMap<ID, SessionID>>,
    users: &UsersGroup,
    report: &mut Report,
) {
    run_session_stress_test(
        report,
        "set_session_info",
        sessions,
        users,
        100,
        100,
        |user, _now, _users, sessions| async move {
            // Get session_id in one lock to avoid deadlock
            let session_id = {
                let u = user.lock().await;
                sessions.get(&u.id).map(|s| *s)
            };
            if let Some(session_id) = session_id {
                user.lock()
                    .await
                    .oc()
                    .set_session_info(SetSessionInfoRequest {
                        session_id: session_id.0,
                        name: Some(format!("updated_session_{}", rand::random::<u32>())),
                        description: Some("Updated by stress test".to_string()),
                        avatar_key: None,
                    })
                    .await
                    .is_ok()
            } else {
                false
            }
        },
    )
    .await;
}

#[register_test("Invite User to Session", WithSessions)]
pub async fn test_invite_user_to_session(
    sessions: Arc<DashMap<ID, SessionID>>,
    users: &UsersGroup,
    report: &mut Report,
) {
    run_session_stress_test(
        report,
        "invite_user_to_session",
        sessions,
        users,
        100,
        100,
        |user, now, users, sessions| async move {
            // Get user_id, session_id, and invitee_id in one lock to avoid deadlock
            let (session_id, invitee_id) = {
                let u = user.lock().await;
                let session_id = sessions.get(&u.id).map(|s| *s);
                let invitee_idx = (now + 1) % users.len();
                let invitee_id = users[invitee_idx].lock().await.id;
                (session_id, invitee_id)
            };
            if let Some(session_id) = session_id {
                user.lock()
                    .await
                    .oc()
                    .invite_user_to_session(InviteUserToSessionRequest {
                        session_id: session_id.0,
                        invitee: invitee_id.0,
                        leave_message: Some("Stress test invitation".to_string()),
                    })
                    .await
                    .is_ok()
            } else {
                false
            }
        },
    )
    .await;
}

#[register_test("Leave Session", WithSessions)]
pub async fn test_leave_session(
    sessions: Arc<DashMap<ID, SessionID>>,
    users: &UsersGroup,
    report: &mut Report,
) {
    run_session_stress_test(
        report,
        "leave_session",
        sessions,
        users,
        100,
        100,
        |user, _now, _users, sessions| async move {
            // Get session_id in one lock to avoid deadlock
            let session_id = {
                let u = user.lock().await;
                sessions.get(&u.id).map(|s| *s)
            };
            if let Some(session_id) = session_id {
                user.lock()
                    .await
                    .oc()
                    .leave_session(LeaveSessionRequest {
                        session_id: session_id.0,
                    })
                    .await
                    .is_ok()
            } else {
                false
            }
        },
    )
    .await;
}

#[register_test("Delete Session", WithSessions)]
pub async fn test_delete_session(
    sessions: Arc<DashMap<ID, SessionID>>,
    users: &UsersGroup,
    report: &mut Report,
) {
    run_session_stress_test(
        report,
        "delete_session",
        sessions,
        users,
        100,
        100,
        |user, _now, _users, sessions| async move {
            // Get session_id in one lock to avoid deadlock
            let session_id = {
                let u = user.lock().await;
                sessions.get(&u.id).map(|s| *s)
            };
            if let Some(session_id) = session_id {
                user.lock()
                    .await
                    .oc()
                    .delete_session(DeleteSessionRequest {
                        session_id: session_id.0,
                    })
                    .await
                    .is_ok()
            } else {
                false
            }
        },
    )
    .await;
}

// Session moderation tests
#[register_test("Ban User", WithSessions)]
pub async fn test_ban(
    sessions: Arc<DashMap<ID, SessionID>>,
    users: &UsersGroup,
    report: &mut Report,
) {
    run_session_stress_test(
        report,
        "ban",
        sessions,
        users,
        100,
        100,
        |user, now, users, sessions| async move {
            // Get session_id and ban_id in one lock to avoid deadlock
            let (session_id, ban_id) = {
                let u = user.lock().await;
                let session_id = sessions.get(&u.id).map(|s| *s);
                let ban_idx = (now + 1) % users.len();
                let ban_id = users[ban_idx].lock().await.id;
                (session_id, ban_id)
            };
            if let Some(session_id) = session_id {
                user.lock()
                    .await
                    .oc()
                    .ban_user(BanUserRequest {
                        session_id: session_id.0,
                        user_ids: vec![ban_id.0],
                        duration: None,
                    })
                    .await
                    .is_ok()
            } else {
                false
            }
        },
    )
    .await;
}

#[register_test("Mute User", WithSessions)]
pub async fn test_mute(
    sessions: Arc<DashMap<ID, SessionID>>,
    users: &UsersGroup,
    report: &mut Report,
) {
    run_session_stress_test(
        report,
        "mute",
        sessions,
        users,
        100,
        100,
        |user, now, users, sessions| async move {
            // Get session_id and mute_id in one lock to avoid deadlock
            let (session_id, mute_id) = {
                let u = user.lock().await;
                let session_id = sessions.get(&u.id).map(|s| *s);
                let mute_idx = (now + 1) % users.len();
                let mute_id = users[mute_idx].lock().await.id;
                (session_id, mute_id)
            };
            if let Some(session_id) = session_id {
                user.lock()
                    .await
                    .oc()
                    .mute_user(MuteUserRequest {
                        session_id: session_id.0,
                        user_ids: vec![mute_id.0],
                        duration: None,
                    })
                    .await
                    .is_ok()
            } else {
                false
            }
        },
    )
    .await;
}

#[register_test("Kick User", WithSessions)]
pub async fn test_kick(
    sessions: Arc<DashMap<ID, SessionID>>,
    users: &UsersGroup,
    report: &mut Report,
) {
    run_session_stress_test(
        report,
        "kick",
        sessions,
        users,
        100,
        100,
        |user, now, users, sessions| async move {
            // Get session_id and kick_id in one lock to avoid deadlock
            let (session_id, kick_id) = {
                let u = user.lock().await;
                let session_id = sessions.get(&u.id).map(|s| *s);
                let kick_idx = (now + 1) % users.len();
                let kick_id = users[kick_idx].lock().await.id;
                (session_id, kick_id)
            };
            if let Some(session_id) = session_id {
                user.lock()
                    .await
                    .oc()
                    .kick_user(KickUserRequest {
                        session_id: session_id.0,
                        user_id: kick_id.0,
                    })
                    .await
                    .is_ok()
            } else {
                false
            }
        },
    )
    .await;
}

// Session role tests
#[register_test("Add Role", WithSessions)]
pub async fn test_add_role(
    sessions: Arc<DashMap<ID, SessionID>>,
    users: &UsersGroup,
    report: &mut Report,
) -> anyhow::Result<Arc<DashMap<ID, u64>>> {
    tracing::info!("▶️  Running test: 'add_role'");
    let mut stress_test = StressTest::builder().set_concurrency(100).set_requests(100);
    let users = users.clone();
    let idx = Arc::new(AtomicUsize::new(0));
    let role_ids = Arc::new(DashMap::new());
    let role_ids_ret = role_ids.clone();
    let output = stress_test
        .stress_test(move || {
            let now = idx.fetch_add(1, Ordering::Relaxed);
            let user = users[now].clone();
            let sessions = sessions.clone();
            let role_ids = role_ids.clone();
            async move {
                // Get user_id and session_id in one lock to avoid deadlock
                let (user_id, session_id) = {
                    let u = user.lock().await;
                    let user_id = u.id;
                    let session_id = sessions.get(&user_id).map(|s| *s);
                    (user_id, session_id)
                };
                if let Some(session_id) = session_id {
                    match user
                        .lock()
                        .await
                        .oc()
                        .add_role(AddRoleRequest {
                            session_id: session_id.0,
                            name: format!("role_{}", rand::random::<u32>()),
                            description: Some("Test role".to_string()),
                            permissions: vec![1, 2, 3],
                        })
                        .await
                    {
                        Ok(resp) => {
                            let role_id = resp.into_inner().role_id;
                            role_ids.insert(user_id, role_id);
                            true
                        }
                        Err(_) => false,
                    }
                } else {
                    false
                }
            }
        })
        .await;
    report.add_record(Record::new("add_role", output));
    Ok(role_ids_ret)
}

#[register_test("Set Role", WithSessions)]
pub async fn test_set_role(
    sessions: Arc<DashMap<ID, SessionID>>,
    users: &UsersGroup,
    report: &mut Report,
) {
    run_session_stress_test(
        report,
        "set_role",
        sessions,
        users,
        100,
        100,
        |user, now, users, sessions| async move {
            // Get session_id and member_id in one lock to avoid deadlock
            // Use predefined Member role (3) instead of custom role_id
            let (session_id, member_id) = {
                let u = user.lock().await;
                let user_id = u.id;
                let session_id = sessions.get(&user_id).map(|s| *s);
                let member_idx = (now + 1) % users.len();
                let member_id = users[member_idx].lock().await.id;
                (session_id, member_id)
            };
            if let Some(session_id) = session_id {
                user.lock()
                    .await
                    .oc()
                    .set_role(SetRoleRequest {
                        session_id: session_id.0,
                        role_id: PredefinedRoles::Member.into(),
                        member_id: member_id.0,
                    })
                    .await
                    .is_ok()
            } else {
                false
            }
        },
    )
    .await;
}

#[register_test("Get Role", WithSessions)]
pub async fn test_get_role(
    role_ids: Arc<DashMap<ID, u64>>,
    users: &UsersGroup,
    report: &mut Report,
) {
    tracing::info!("▶️  Running test: 'get_role'");
    let mut stress_test = StressTest::builder().set_concurrency(100).set_requests(100);
    let users = users.clone();
    let idx = Arc::new(AtomicUsize::new(0));
    let output = stress_test
        .stress_test(move || {
            let now = idx.fetch_add(1, Ordering::Relaxed);
            let user = users[now].clone();
            let role_ids = role_ids.clone();
            async move {
                // Get user_id and role_id in one lock to avoid deadlock
                let role_id = {
                    let u = user.lock().await;
                    role_ids.get(&u.id).map(|r| *r)
                };
                if let Some(role_id) = role_id {
                    user.lock()
                        .await
                        .oc()
                        .get_role(GetRoleRequest { role_id })
                        .await
                        .is_ok()
                } else {
                    false
                }
            }
        })
        .await;
    report.add_record(Record::new("get_role", output));
}

// Additional session tests
#[register_test("Join Session", WithSessions)]
pub async fn test_join_session(
    sessions: Arc<DashMap<ID, SessionID>>,
    users: &UsersGroup,
    report: &mut Report,
) {
    run_session_stress_test(
        report,
        "join_session",
        sessions,
        users,
        100,
        100,
        |user, now, _users, sessions| async move {
            // Get user_id and find session in one lock to avoid deadlock
            let (user_id, found_session) = {
                let u = user.lock().await;
                let user_id = u.id;
                // Try to join another user's session
                let session_count = sessions.len();
                if session_count == 0 {
                    return false;
                }
                let session_idx = (now + 1) % session_count;
                let found_session = sessions
                    .iter()
                    .nth(session_idx)
                    .map(|entry| (*entry.key(), *entry.value()));
                (user_id, found_session)
            };
            if let Some((session_owner_id, session_id)) = found_session {
                if session_owner_id != user_id {
                    user.lock()
                        .await
                        .oc()
                        .join_session(JoinSessionRequest {
                            session_id: session_id.0,
                            leave_message: Some("Stress test join".to_string()),
                        })
                        .await
                        .is_ok()
                } else {
                    // Skip if it's our own session
                    true
                }
            } else {
                false
            }
        },
    )
    .await;
}

#[register_test("Accept Join Session Invitation", WithSessions)]
pub async fn test_accept_join_session_invitation(
    sessions: Arc<DashMap<ID, SessionID>>,
    users: &UsersGroup,
    report: &mut Report,
) {
    run_session_stress_test(
        report,
        "accept_join_session_invitation",
        sessions,
        users,
        100,
        100,
        |_user, now, users, sessions| async move {
            // First send an invitation, then accept it
            // Get inviter (session owner) and invitee
            let (inviter, session_id, invitee_id) = {
                let inviter_idx = (now + 1) % users.len();
                let inviter = users[inviter_idx].clone();
                let inviter_id = inviter.lock().await.id;
                let session_id = sessions.get(&inviter_id).map(|s| *s);
                let invitee_idx = now;
                let invitee_id = users[invitee_idx].lock().await.id;
                (inviter, session_id, invitee_id)
            };

            if let Some(session_id) = session_id {
                // First send invitation
                let invite_result = inviter
                    .lock()
                    .await
                    .oc()
                    .invite_user_to_session(InviteUserToSessionRequest {
                        session_id: session_id.0,
                        invitee: invitee_id.0,
                        leave_message: Some("Please join my session".to_string()),
                    })
                    .await;

                if invite_result.is_ok() {
                    // Then accept it
                    let user = users[now].clone();
                    user.lock()
                        .await
                        .oc()
                        .accept_join_session_invitation(AcceptJoinSessionInvitationRequest {
                            session_id: session_id.0,
                            accepted: true,
                            inviter_id: inviter.lock().await.id.0,
                        })
                        .await
                        .is_ok()
                } else {
                    false
                }
            } else {
                false
            }
        },
    )
    .await;
}

#[register_test("Allow User Join Session", WithSessions)]
pub async fn test_allow_user_join_session(
    sessions: Arc<DashMap<ID, SessionID>>,
    users: &UsersGroup,
    report: &mut Report,
) {
    run_session_stress_test(
        report,
        "allow_user_join_session",
        sessions,
        users,
        100,
        100,
        |user, now, users, sessions| async move {
            // Get user_id, session_id, and requester_id in one lock to avoid deadlock
            let (session_id, requester_id) = {
                let u = user.lock().await;
                let user_id = u.id;
                let session_id = sessions.get(&user_id).map(|s| *s);
                let requester_idx = (now + 1) % users.len();
                let requester_id = users[requester_idx].lock().await.id;
                (session_id, requester_id)
            };
            if let Some(session_id) = session_id {
                user.lock()
                    .await
                    .oc()
                    .allow_user_join_session(AllowUserJoinSessionRequest {
                        session_id: session_id.0,
                        user_id: requester_id.0,
                        accepted: true,
                        room_key: None,
                    })
                    .await
                    .is_ok()
            } else {
                false
            }
        },
    )
    .await;
}

#[register_test("E2EEize Session", WithSessions)]
pub async fn test_e2eeize_session(
    sessions: Arc<DashMap<ID, SessionID>>,
    users: &UsersGroup,
    report: &mut Report,
) {
    run_session_stress_test(
        report,
        "e2eeize_session",
        sessions,
        users,
        100,
        100,
        |_user, _now, _users, sessions| async move {
            let user = _users[0].clone();
            // Get session_id in one lock to avoid deadlock
            let session_id = {
                let u = user.lock().await;
                sessions.get(&u.id).map(|s| *s)
            };
            if let Some(session_id) = session_id {
                user.lock()
                    .await
                    .oc()
                    .e2eeize_session(E2eeizeSessionRequest {
                        session_id: session_id.0,
                    })
                    .await
                    .is_ok()
            } else {
                false
            }
        },
    )
    .await;
}

#[register_test("Dee2eeize Session", WithSessions)]
pub async fn test_dee2eeize_session(
    sessions: Arc<DashMap<ID, SessionID>>,
    users: &UsersGroup,
    report: &mut Report,
) {
    run_session_stress_test(
        report,
        "dee2eeize_session",
        sessions,
        users,
        100,
        100,
        |_user, _now, _users, sessions| async move {
            let user = _users[0].clone();
            // Get session_id in one lock to avoid deadlock
            let session_id = {
                let u = user.lock().await;
                sessions.get(&u.id).map(|s| *s)
            };
            if let Some(session_id) = session_id {
                user.lock()
                    .await
                    .oc()
                    .dee2eeize_session(Dee2eeizeSessionRequest {
                        session_id: session_id.0,
                    })
                    .await
                    .is_ok()
            } else {
                false
            }
        },
    )
    .await;
}

#[register_test("Send Room Key", WithSessions)]
pub async fn test_send_room_key(
    sessions: Arc<DashMap<ID, SessionID>>,
    users: &UsersGroup,
    report: &mut Report,
) {
    use bytes::Bytes;
    run_session_stress_test(
        report,
        "send_room_key",
        sessions,
        users,
        100,
        100,
        |user, now, users, sessions| async move {
            // Get session_id and recipient_id in one lock to avoid deadlock
            let (session_id, recipient_id) = {
                let u = user.lock().await;
                let user_id = u.id;
                let session_id = sessions.get(&user_id).map(|s| *s);
                let recipient_idx = (now + 1) % users.len();
                let recipient_id = users[recipient_idx].lock().await.id;
                (session_id, recipient_id)
            };
            if let Some(session_id) = session_id {
                user.lock()
                    .await
                    .oc()
                    .send_room_key(SendRoomKeyRequest {
                        session_id: session_id.0,
                        user_id: recipient_id.0,
                        room_key: Bytes::from(vec![1, 2, 3, 4]), // Dummy key
                    })
                    .await
                    .is_ok()
            } else {
                false
            }
        },
    )
    .await;
}
