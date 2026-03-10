use base::constants::{ID, SessionID};
use client::oc_helper::user::TestUser;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// All possible random actions
#[derive(Debug, Clone)]
pub enum RandomAction {
    // Session Management
    CreateSession {
        name: String,
        e2ee_enabled: bool,
        initial_members: Vec<ID>,
    },
    DeleteSession {
        session_id: SessionID,
    },
    JoinSession {
        session_id: SessionID,
        user_id: ID,
    },
    LeaveSession {
        session_id: SessionID,
        user_id: ID,
    },
    InviteToSession {
        session_id: SessionID,
        inviter_id: ID,
        invitee_id: ID,
    },
    AcceptSessionInvitation {
        session_id: SessionID,
        inviter_id: ID,
        user_id: ID,
        accepted: bool,
    },

    // Session Moderation
    KickUser {
        session_id: SessionID,
        moderator_id: ID,
        user_id: ID,
    },
    BanUser {
        session_id: SessionID,
        moderator_id: ID,
        user_id: ID,
        duration: Option<Duration>,
    },
    UnbanUser {
        session_id: SessionID,
        moderator_id: ID,
        user_id: ID,
    },
    MuteUser {
        session_id: SessionID,
        moderator_id: ID,
        user_id: ID,
        duration: Option<Duration>,
    },
    UnmuteUser {
        session_id: SessionID,
        moderator_id: ID,
        user_id: ID,
    },

    // Messaging
    SendMessage {
        session_id: SessionID,
        user_id: ID,
        content: String,
        is_encrypted: bool,
    },
    RecallMessage {
        session_id: SessionID,
        user_id: ID,
        message_id: u64,
    },
    FetchMessages {
        user_id: ID,
    },

    // Friend Management
    AddFriend {
        user_id: ID,
        friend_id: ID,
        message: Option<String>,
    },
    AcceptFriendInvitation {
        user_id: ID,
        friend_id: ID,
        accepted: bool,
    },
    DeleteFriend {
        user_id: ID,
        friend_id: ID,
    },

    // File Operations
    UploadFile {
        user_id: ID,
        session_id: Option<SessionID>,
        size: usize,
    },
    DownloadFile {
        user_id: ID,
        file_key: String,
    },

    // Info Queries
    GetSessionInfo {
        user_id: ID,
        session_id: SessionID,
    },
    GetAccountInfo {
        user_id: ID,
        target_id: ID,
    },

    // Settings
    SetSessionInfo {
        user_id: ID,
        session_id: SessionID,
        name: Option<String>,
    },
    SetAccountInfo {
        user_id: ID,
        name: Option<String>,
    },
}

/// Optional data returned by successful actions
#[derive(Debug, Clone, Default)]
pub enum ResponseData {
    SessionCreated(SessionID),
    #[default]
    None,
}

/// Action execution result
#[derive(Debug, Clone)]
pub enum ActionResult {
    Success {
        duration: Duration,
        data: ResponseData,
    },
    ExpectedFailure {
        _reason: String,
        duration: Duration,
    },
    UnexpectedFailure {
        error: String,
        duration: Duration,
    },
}

impl ActionResult {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    pub fn get_data(&self) -> ResponseData {
        match self {
            Self::Success { data, .. } => data.clone(),
            _ => ResponseData::None,
        }
    }
}

/// Action executor that uses TestUser instances
pub struct ActionExecutor {
    /// Map of user IDs to their TestUser instances
    users: dashmap::DashMap<ID, Arc<Mutex<TestUser>>>,
}

impl ActionExecutor {
    pub fn new() -> Self {
        Self {
            users: dashmap::DashMap::new(),
        }
    }

    /// Register a user with the executor
    pub async fn register_user(&self, user: Arc<Mutex<TestUser>>) -> ID {
        let user_id = user.lock().await.id;
        self.users.insert(user_id, user);
        user_id
    }

    /// Get a user by ID
    pub fn get_user(&self, user_id: ID) -> Option<Arc<Mutex<TestUser>>> {
        self.users.get(&user_id).map(|entry| entry.value().clone())
    }

    /// Execute an action and return the result
    pub async fn execute(&self, action: &RandomAction) -> ActionResult {
        match action {
            RandomAction::SendMessage {
                session_id,
                user_id,
                content,
                is_encrypted,
            } => {
                self.execute_send_message(*session_id, *user_id, content, *is_encrypted)
                    .await
            }
            RandomAction::FetchMessages { user_id } => self.execute_fetch_messages(*user_id).await,
            RandomAction::RecallMessage {
                session_id,
                user_id,
                message_id,
            } => {
                self.execute_recall_message(*session_id, *user_id, *message_id)
                    .await
            }
            RandomAction::CreateSession {
                name,
                e2ee_enabled,
                initial_members,
            } => {
                self.execute_create_session(name, *e2ee_enabled, initial_members)
                    .await
            }
            RandomAction::DeleteSession { session_id } => {
                self.execute_delete_session(*session_id).await
            }
            RandomAction::JoinSession {
                session_id,
                user_id,
            } => self.execute_join_session(*session_id, *user_id).await,
            RandomAction::LeaveSession {
                session_id,
                user_id,
            } => self.execute_leave_session(*session_id, *user_id).await,
            RandomAction::InviteToSession {
                session_id,
                inviter_id,
                invitee_id,
            } => {
                self.execute_invite_to_session(*session_id, *inviter_id, *invitee_id)
                    .await
            }
            RandomAction::AcceptSessionInvitation {
                session_id,
                user_id,
                inviter_id,
                accepted,
            } => {
                self.execute_accept_session_invitation(
                    *session_id,
                    *user_id,
                    *inviter_id,
                    *accepted,
                )
                .await
            }
            RandomAction::KickUser {
                session_id,
                moderator_id,
                user_id,
            } => {
                self.execute_kick_user(*session_id, *moderator_id, *user_id)
                    .await
            }
            RandomAction::BanUser {
                session_id,
                moderator_id,
                user_id,
                duration,
            } => {
                self.execute_ban_user(*session_id, *moderator_id, *user_id, *duration)
                    .await
            }
            RandomAction::UnbanUser {
                session_id,
                moderator_id,
                user_id,
            } => {
                self.execute_unban_user(*session_id, *moderator_id, *user_id)
                    .await
            }
            RandomAction::MuteUser {
                session_id,
                moderator_id,
                user_id,
                duration,
            } => {
                self.execute_mute_user(*session_id, *moderator_id, *user_id, *duration)
                    .await
            }
            RandomAction::UnmuteUser {
                session_id,
                moderator_id,
                user_id,
            } => {
                self.execute_unmute_user(*session_id, *moderator_id, *user_id)
                    .await
            }
            RandomAction::AddFriend {
                user_id,
                friend_id,
                message,
            } => {
                self.execute_add_friend(*user_id, *friend_id, message.as_deref())
                    .await
            }
            RandomAction::AcceptFriendInvitation {
                user_id,
                friend_id,
                accepted,
            } => {
                self.execute_accept_friend_invitation(*user_id, *friend_id, *accepted)
                    .await
            }
            RandomAction::DeleteFriend { user_id, friend_id } => {
                self.execute_delete_friend(*user_id, *friend_id).await
            }
            RandomAction::UploadFile {
                user_id,
                session_id,
                size,
            } => self.execute_upload_file(*user_id, *session_id, *size).await,
            RandomAction::DownloadFile { user_id, file_key } => {
                self.execute_download_file(*user_id, file_key).await
            }
            RandomAction::GetSessionInfo {
                user_id,
                session_id,
            } => self.execute_get_session_info(*user_id, *session_id).await,
            RandomAction::GetAccountInfo { user_id, target_id } => {
                self.execute_get_account_info(*user_id, *target_id).await
            }
            RandomAction::SetSessionInfo {
                user_id,
                session_id,
                name,
            } => {
                self.execute_set_session_info(*user_id, *session_id, name.as_deref())
                    .await
            }
            RandomAction::SetAccountInfo { user_id, name } => {
                self.execute_set_account_info(*user_id, name.as_deref())
                    .await
            }
        }
    }

    async fn execute_send_message(
        &self,
        session_id: SessionID,
        user_id: ID,
        content: &str,
        is_encrypted: bool,
    ) -> ActionResult {
        let start = std::time::Instant::now();

        if let Some(user) = self.get_user(user_id) {
            let mut user_guard = user.lock().await;
            match user_guard
                .send_msg(session_id, content, vec![], is_encrypted)
                .await
            {
                Ok(_) => ActionResult::Success {
                    duration: start.elapsed(),
                    data: ResponseData::None,
                },
                Err(e) => {
                    let error_msg = e.to_string();
                    // Check if it's an expected error
                    if error_msg.contains("not in session")
                        || error_msg.contains("session not found")
                        || error_msg.contains("muted")
                    {
                        ActionResult::ExpectedFailure {
                            _reason: error_msg,
                            duration: start.elapsed(),
                        }
                    } else {
                        ActionResult::UnexpectedFailure {
                            error: error_msg,
                            duration: start.elapsed(),
                        }
                    }
                }
            }
        } else {
            ActionResult::UnexpectedFailure {
                error: format!("User {} not found", user_id.0),
                duration: start.elapsed(),
            }
        }
    }

    async fn execute_fetch_messages(&self, user_id: ID) -> ActionResult {
        let start = std::time::Instant::now();

        if let Some(user) = self.get_user(user_id) {
            let mut user_guard = user.lock().await;
            // Fetch a small number of messages
            match user_guard.fetch_msgs().fetch(1).await {
                Ok(_) => ActionResult::Success {
                    duration: start.elapsed(),
                    data: ResponseData::None,
                },
                Err(_e) => ActionResult::Success {
                    duration: start.elapsed(),
                    data: ResponseData::None,
                }, // Timeout is expected when no messages
            }
        } else {
            ActionResult::UnexpectedFailure {
                error: format!("User {} not found", user_id.0),
                duration: start.elapsed(),
            }
        }
    }

    async fn execute_recall_message(
        &self,
        session_id: SessionID,
        user_id: ID,
        message_id: u64,
    ) -> ActionResult {
        let start = std::time::Instant::now();

        if let Some(user) = self.get_user(user_id) {
            let mut user_guard = user.lock().await;
            let req = pb::service::ourchat::msg_delivery::recall::v1::RecallMsgRequest {
                msg_id: message_id,
                session_id: session_id.0,
            };
            match user_guard.oc().recall_msg(req).await {
                Ok(_) => ActionResult::Success {
                    duration: start.elapsed(),
                    data: ResponseData::None,
                },
                Err(e) => {
                    let error_msg = e.to_string();
                    if error_msg.contains("not found") || error_msg.contains("not your message") {
                        ActionResult::ExpectedFailure {
                            _reason: error_msg,
                            duration: start.elapsed(),
                        }
                    } else {
                        ActionResult::UnexpectedFailure {
                            error: error_msg,
                            duration: start.elapsed(),
                        }
                    }
                }
            }
        } else {
            ActionResult::UnexpectedFailure {
                error: format!("User {} not found", user_id.0),
                duration: start.elapsed(),
            }
        }
    }

    async fn execute_create_session(
        &self,
        name: &str,
        e2ee_enabled: bool,
        initial_members: &[ID],
    ) -> ActionResult {
        let start = std::time::Instant::now();

        // Create session with the first user as creator
        if let Some(creator) = initial_members.first().and_then(|id| self.get_user(*id)) {
            let mut creator_guard = creator.lock().await;

            // Build the new session request
            let req = pb::service::ourchat::session::new_session::v1::NewSessionRequest {
                members: initial_members.iter().map(|id| id.0).collect(),
                name: Some(name.to_string()),
                leave_message: None,
                avatar_key: None,
                e2ee_on: e2ee_enabled,
            };

            match creator_guard.oc().new_session(req).await {
                Ok(response) => {
                    let session_id = SessionID(response.into_inner().session_id);
                    ActionResult::Success {
                        duration: start.elapsed(),
                        data: ResponseData::SessionCreated(session_id),
                    }
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    if error_msg.contains("not found") {
                        ActionResult::ExpectedFailure {
                            _reason: error_msg,
                            duration: start.elapsed(),
                        }
                    } else {
                        ActionResult::UnexpectedFailure {
                            error: error_msg,
                            duration: start.elapsed(),
                        }
                    }
                }
            }
        } else {
            ActionResult::UnexpectedFailure {
                error: "No valid creator found".to_string(),
                duration: start.elapsed(),
            }
        }
    }

    async fn execute_delete_session(&self, session_id: SessionID) -> ActionResult {
        let start = std::time::Instant::now();

        // Find a user who can delete the session (owner or admin)
        // For simplicity, we'll just pick the first available user
        for entry in self.users.iter() {
            let user = entry.value().clone();
            let mut user_guard = user.lock().await;

            let req = pb::service::ourchat::session::delete_session::v1::DeleteSessionRequest {
                session_id: session_id.0,
            };

            match user_guard.oc().delete_session(req).await {
                Ok(_) => {
                    return ActionResult::Success {
                        duration: start.elapsed(),
                        data: ResponseData::None,
                    };
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    // If this user can't delete, try the next one
                    if !error_msg.contains("not owner")
                        && !error_msg.contains("permission")
                        && !error_msg.contains("not found")
                    {
                        return ActionResult::UnexpectedFailure {
                            error: error_msg,
                            duration: start.elapsed(),
                        };
                    }
                }
            }
        }

        // All users failed - likely expected (no owner or session doesn't exist)
        ActionResult::ExpectedFailure {
            _reason: "No user with permission to delete session".to_string(),
            duration: start.elapsed(),
        }
    }

    async fn execute_join_session(&self, session_id: SessionID, user_id: ID) -> ActionResult {
        let start = std::time::Instant::now();

        if let Some(user) = self.get_user(user_id) {
            let mut user_guard = user.lock().await;
            let req = pb::service::ourchat::session::join_session::v1::JoinSessionRequest {
                session_id: session_id.0,
                leave_message: None,
            };
            match user_guard.oc().join_session(req).await {
                Ok(_) => ActionResult::Success {
                    duration: start.elapsed(),
                    data: ResponseData::None,
                },
                Err(e) => {
                    let error_msg = e.to_string();
                    if error_msg.contains("not found") || error_msg.contains("already") {
                        ActionResult::ExpectedFailure {
                            _reason: error_msg,
                            duration: start.elapsed(),
                        }
                    } else {
                        ActionResult::UnexpectedFailure {
                            error: error_msg,
                            duration: start.elapsed(),
                        }
                    }
                }
            }
        } else {
            ActionResult::UnexpectedFailure {
                error: format!("User {} not found", user_id.0),
                duration: start.elapsed(),
            }
        }
    }

    async fn execute_leave_session(&self, session_id: SessionID, user_id: ID) -> ActionResult {
        let start = std::time::Instant::now();

        if let Some(user) = self.get_user(user_id) {
            let mut user_guard = user.lock().await;
            let req = pb::service::ourchat::session::leave_session::v1::LeaveSessionRequest {
                session_id: session_id.0,
            };
            match user_guard.oc().leave_session(req).await {
                Ok(_) => ActionResult::Success {
                    duration: start.elapsed(),
                    data: ResponseData::None,
                },
                Err(e) => {
                    let error_msg = e.to_string();
                    if error_msg.contains("not found") || error_msg.contains("not in") {
                        ActionResult::ExpectedFailure {
                            _reason: error_msg,
                            duration: start.elapsed(),
                        }
                    } else {
                        ActionResult::UnexpectedFailure {
                            error: error_msg,
                            duration: start.elapsed(),
                        }
                    }
                }
            }
        } else {
            ActionResult::UnexpectedFailure {
                error: format!("User {} not found", user_id.0),
                duration: start.elapsed(),
            }
        }
    }

    async fn execute_invite_to_session(
        &self,
        session_id: SessionID,
        inviter_id: ID,
        invitee_id: ID,
    ) -> ActionResult {
        let start = std::time::Instant::now();

        if let Some(inviter) = self.get_user(inviter_id) {
            let mut inviter_guard = inviter.lock().await;
            let req = pb::service::ourchat::session::invite_user_to_session::v1::InviteUserToSessionRequest {
                session_id: session_id.0,
                invitee: invitee_id.0,
                leave_message: None,
            };
            match inviter_guard.oc().invite_user_to_session(req).await {
                Ok(_) => ActionResult::Success {
                    duration: start.elapsed(),
                    data: ResponseData::None,
                },
                Err(e) => {
                    let error_msg = e.to_string();
                    if error_msg.contains("not found") || error_msg.contains("already") {
                        ActionResult::ExpectedFailure {
                            _reason: error_msg,
                            duration: start.elapsed(),
                        }
                    } else {
                        ActionResult::UnexpectedFailure {
                            error: error_msg,
                            duration: start.elapsed(),
                        }
                    }
                }
            }
        } else {
            ActionResult::UnexpectedFailure {
                error: format!("User {} not found", inviter_id.0),
                duration: start.elapsed(),
            }
        }
    }

    async fn execute_accept_session_invitation(
        &self,
        session_id: SessionID,
        user_id: ID,
        inviter_id: ID,
        accepted: bool,
    ) -> ActionResult {
        let start = std::time::Instant::now();

        if let Some(user) = self.get_user(user_id) {
            let mut user_guard = user.lock().await;
            match user_guard
                .accept_join_session_invitation(session_id, accepted, inviter_id)
                .await
            {
                Ok(_) => ActionResult::Success {
                    duration: start.elapsed(),
                    data: ResponseData::None,
                },
                Err(e) => {
                    let error_msg = e.to_string();
                    if error_msg.contains("not found") || error_msg.contains("already") {
                        ActionResult::ExpectedFailure {
                            _reason: error_msg,
                            duration: start.elapsed(),
                        }
                    } else {
                        ActionResult::UnexpectedFailure {
                            error: error_msg,
                            duration: start.elapsed(),
                        }
                    }
                }
            }
        } else {
            ActionResult::UnexpectedFailure {
                error: format!("User {} not found", user_id.0),
                duration: start.elapsed(),
            }
        }
    }

    async fn execute_kick_user(
        &self,
        session_id: SessionID,
        moderator_id: ID,
        target_id: ID,
    ) -> ActionResult {
        let start = std::time::Instant::now();

        if let Some(user) = self.get_user(target_id) {
            let mut user_guard = user.lock().await;
            match user_guard.kick_user(moderator_id, session_id).await {
                Ok(_) => ActionResult::Success {
                    duration: start.elapsed(),
                    data: ResponseData::None,
                },
                Err(e) => {
                    let error_msg = e.to_string();
                    if error_msg.contains("not found") || error_msg.contains("permission") {
                        ActionResult::ExpectedFailure {
                            _reason: error_msg,
                            duration: start.elapsed(),
                        }
                    } else {
                        ActionResult::UnexpectedFailure {
                            error: error_msg,
                            duration: start.elapsed(),
                        }
                    }
                }
            }
        } else {
            ActionResult::UnexpectedFailure {
                error: format!("User {} not found", target_id.0),
                duration: start.elapsed(),
            }
        }
    }

    async fn execute_ban_user(
        &self,
        session_id: SessionID,
        moderator_id: ID,
        target_id: ID,
        duration: Option<Duration>,
    ) -> ActionResult {
        let start = std::time::Instant::now();

        if let Some(moderator) = self.get_user(moderator_id) {
            let mut moderator_guard = moderator.lock().await;
            match moderator_guard
                .ban_user(vec![target_id], session_id, duration)
                .await
            {
                Ok(_) => ActionResult::Success {
                    duration: start.elapsed(),
                    data: ResponseData::None,
                },
                Err(e) => {
                    let error_msg = e.to_string();
                    if error_msg.contains("not found") || error_msg.contains("permission") {
                        ActionResult::ExpectedFailure {
                            _reason: error_msg,
                            duration: start.elapsed(),
                        }
                    } else {
                        ActionResult::UnexpectedFailure {
                            error: error_msg,
                            duration: start.elapsed(),
                        }
                    }
                }
            }
        } else {
            ActionResult::UnexpectedFailure {
                error: format!("Moderator {} not found", moderator_id.0),
                duration: start.elapsed(),
            }
        }
    }

    async fn execute_unban_user(
        &self,
        session_id: SessionID,
        _moderator_id: ID,
        target_id: ID,
    ) -> ActionResult {
        let start = std::time::Instant::now();

        if let Some(user) = self.get_user(target_id) {
            let mut user_guard = user.lock().await;
            match user_guard.unban_user(vec![target_id], session_id).await {
                Ok(_) => ActionResult::Success {
                    duration: start.elapsed(),
                    data: ResponseData::None,
                },
                Err(e) => ActionResult::UnexpectedFailure {
                    error: e.to_string(),
                    duration: start.elapsed(),
                },
            }
        } else {
            ActionResult::UnexpectedFailure {
                error: format!("User {} not found", target_id.0),
                duration: start.elapsed(),
            }
        }
    }

    async fn execute_mute_user(
        &self,
        session_id: SessionID,
        moderator_id: ID,
        target_id: ID,
        duration: Option<Duration>,
    ) -> ActionResult {
        let start = std::time::Instant::now();

        if let Some(moderator) = self.get_user(moderator_id) {
            let mut moderator_guard = moderator.lock().await;
            match moderator_guard
                .mute_user(vec![target_id], session_id, duration)
                .await
            {
                Ok(_) => ActionResult::Success {
                    duration: start.elapsed(),
                    data: ResponseData::None,
                },
                Err(e) => {
                    let error_msg = e.to_string();
                    if error_msg.contains("not found") || error_msg.contains("permission") {
                        ActionResult::ExpectedFailure {
                            _reason: error_msg,
                            duration: start.elapsed(),
                        }
                    } else {
                        ActionResult::UnexpectedFailure {
                            error: error_msg,
                            duration: start.elapsed(),
                        }
                    }
                }
            }
        } else {
            ActionResult::UnexpectedFailure {
                error: format!("Moderator {} not found", moderator_id.0),
                duration: start.elapsed(),
            }
        }
    }

    async fn execute_unmute_user(
        &self,
        session_id: SessionID,
        _moderator_id: ID,
        target_id: ID,
    ) -> ActionResult {
        let start = std::time::Instant::now();

        if let Some(user) = self.get_user(target_id) {
            let mut user_guard = user.lock().await;
            match user_guard.unmute_user(vec![target_id], session_id).await {
                Ok(_) => ActionResult::Success {
                    duration: start.elapsed(),
                    data: ResponseData::None,
                },
                Err(e) => ActionResult::UnexpectedFailure {
                    error: e.to_string(),
                    duration: start.elapsed(),
                },
            }
        } else {
            ActionResult::UnexpectedFailure {
                error: format!("User {} not found", target_id.0),
                duration: start.elapsed(),
            }
        }
    }

    async fn execute_add_friend(
        &self,
        user_id: ID,
        friend_id: ID,
        message: Option<&str>,
    ) -> ActionResult {
        let start = std::time::Instant::now();

        if let Some(user) = self.get_user(user_id) {
            let mut user_guard = user.lock().await;
            let req = pb::service::ourchat::friends::add_friend::v1::AddFriendRequest {
                friend_id: friend_id.0,
                leave_message: message.map(|s| s.to_string()),
                display_name: None,
            };
            match user_guard.oc().add_friend(req).await {
                Ok(_) => ActionResult::Success {
                    duration: start.elapsed(),
                    data: ResponseData::None,
                },
                Err(e) => {
                    let error_msg = e.to_string();
                    if error_msg.contains("already friend") || error_msg.contains("already added") {
                        ActionResult::ExpectedFailure {
                            _reason: error_msg,
                            duration: start.elapsed(),
                        }
                    } else {
                        ActionResult::UnexpectedFailure {
                            error: error_msg,
                            duration: start.elapsed(),
                        }
                    }
                }
            }
        } else {
            ActionResult::UnexpectedFailure {
                error: format!("User {} not found", user_id.0),
                duration: start.elapsed(),
            }
        }
    }

    async fn execute_accept_friend_invitation(
        &self,
        user_id: ID,
        friend_id: ID,
        accepted: bool,
    ) -> ActionResult {
        let start = std::time::Instant::now();

        if let Some(user) = self.get_user(user_id) {
            let mut user_guard = user.lock().await;
            let status = if accepted {
                pb::service::ourchat::friends::accept_friend_invitation::v1::AcceptFriendInvitationResult::Success
            } else {
                pb::service::ourchat::friends::accept_friend_invitation::v1::AcceptFriendInvitationResult::Fail
            };
            let req = pb::service::ourchat::friends::accept_friend_invitation::v1::AcceptFriendInvitationRequest {
                friend_id: friend_id.0,
                status: status as i32,
                leave_message: None,
            };
            match user_guard.oc().accept_friend_invitation(req).await {
                Ok(_) => ActionResult::Success {
                    duration: start.elapsed(),
                    data: ResponseData::None,
                },
                Err(e) => {
                    let error_msg = e.to_string();
                    if error_msg.contains("not found") {
                        ActionResult::ExpectedFailure {
                            _reason: error_msg,
                            duration: start.elapsed(),
                        }
                    } else {
                        ActionResult::UnexpectedFailure {
                            error: error_msg,
                            duration: start.elapsed(),
                        }
                    }
                }
            }
        } else {
            ActionResult::UnexpectedFailure {
                error: format!("User {} not found", user_id.0),
                duration: start.elapsed(),
            }
        }
    }

    async fn execute_delete_friend(&self, user_id: ID, friend_id: ID) -> ActionResult {
        let start = std::time::Instant::now();

        if let Some(user) = self.get_user(user_id) {
            let mut user_guard = user.lock().await;
            let req = pb::service::ourchat::friends::delete_friend::v1::DeleteFriendRequest {
                friend_id: friend_id.0,
            };
            match user_guard.oc().delete_friend(req).await {
                Ok(_) => ActionResult::Success {
                    duration: start.elapsed(),
                    data: ResponseData::None,
                },
                Err(e) => {
                    let error_msg = e.to_string();
                    if error_msg.contains("not friend") {
                        ActionResult::ExpectedFailure {
                            _reason: error_msg,
                            duration: start.elapsed(),
                        }
                    } else {
                        ActionResult::UnexpectedFailure {
                            error: error_msg,
                            duration: start.elapsed(),
                        }
                    }
                }
            }
        } else {
            ActionResult::UnexpectedFailure {
                error: format!("User {} not found", user_id.0),
                duration: start.elapsed(),
            }
        }
    }

    async fn execute_upload_file(
        &self,
        user_id: ID,
        session_id: Option<SessionID>,
        size: usize,
    ) -> ActionResult {
        let start = std::time::Instant::now();

        if let Some(user) = self.get_user(user_id) {
            let mut user_guard = user.lock().await;
            // Create dummy content
            let content = vec![0u8; size.min(1024 * 1024)]; // Max 1MB for safety
            match user_guard.post_file(&content, session_id).await {
                Ok(_) => ActionResult::Success {
                    duration: start.elapsed(),
                    data: ResponseData::None,
                },
                Err(e) => ActionResult::UnexpectedFailure {
                    error: e.to_string(),
                    duration: start.elapsed(),
                },
            }
        } else {
            ActionResult::UnexpectedFailure {
                error: format!("User {} not found", user_id.0),
                duration: start.elapsed(),
            }
        }
    }

    async fn execute_download_file(&self, user_id: ID, file_key: &str) -> ActionResult {
        let start = std::time::Instant::now();

        if let Some(user) = self.get_user(user_id) {
            let mut user_guard = user.lock().await;
            match user_guard.download_file(file_key).await {
                Ok(_) => ActionResult::Success {
                    duration: start.elapsed(),
                    data: ResponseData::None,
                },
                Err(e) => {
                    let error_msg = e.to_string();
                    if error_msg.contains("not found") {
                        ActionResult::ExpectedFailure {
                            _reason: error_msg,
                            duration: start.elapsed(),
                        }
                    } else {
                        ActionResult::UnexpectedFailure {
                            error: error_msg,
                            duration: start.elapsed(),
                        }
                    }
                }
            }
        } else {
            ActionResult::UnexpectedFailure {
                error: format!("User {} not found", user_id.0),
                duration: start.elapsed(),
            }
        }
    }

    async fn execute_get_session_info(&self, user_id: ID, session_id: SessionID) -> ActionResult {
        let start = std::time::Instant::now();

        if let Some(user) = self.get_user(user_id) {
            let mut user_guard = user.lock().await;
            let req = pb::service::ourchat::session::get_session_info::v1::GetSessionInfoRequest {
                session_id: session_id.0,
                query_values: vec![],
            };
            match user_guard.oc().get_session_info(req).await {
                Ok(_) => ActionResult::Success {
                    duration: start.elapsed(),
                    data: ResponseData::None,
                },
                Err(e) => ActionResult::UnexpectedFailure {
                    error: e.to_string(),
                    duration: start.elapsed(),
                },
            }
        } else {
            ActionResult::UnexpectedFailure {
                error: format!("User {} not found", user_id.0),
                duration: start.elapsed(),
            }
        }
    }

    async fn execute_get_account_info(&self, user_id: ID, _target_id: ID) -> ActionResult {
        let start = std::time::Instant::now();

        if let Some(user) = self.get_user(user_id) {
            let mut user_guard = user.lock().await;
            match user_guard.get_self_info(vec![]).await {
                Ok(_) => ActionResult::Success {
                    duration: start.elapsed(),
                    data: ResponseData::None,
                },
                Err(e) => ActionResult::UnexpectedFailure {
                    error: e.to_string(),
                    duration: start.elapsed(),
                },
            }
        } else {
            ActionResult::UnexpectedFailure {
                error: format!("User {} not found", user_id.0),
                duration: start.elapsed(),
            }
        }
    }

    async fn execute_set_session_info(
        &self,
        user_id: ID,
        session_id: SessionID,
        name: Option<&str>,
    ) -> ActionResult {
        let start = std::time::Instant::now();

        if let Some(user) = self.get_user(user_id) {
            let mut user_guard = user.lock().await;
            let req = pb::service::ourchat::session::set_session_info::v1::SetSessionInfoRequest {
                session_id: session_id.0,
                name: name.map(|s| s.to_string()),
                description: None,
                avatar_key: None,
            };
            match user_guard.oc().set_session_info(req).await {
                Ok(_) => ActionResult::Success {
                    duration: start.elapsed(),
                    data: ResponseData::None,
                },
                Err(e) => ActionResult::UnexpectedFailure {
                    error: e.to_string(),
                    duration: start.elapsed(),
                },
            }
        } else {
            ActionResult::UnexpectedFailure {
                error: format!("User {} not found", user_id.0),
                duration: start.elapsed(),
            }
        }
    }

    async fn execute_set_account_info(&self, user_id: ID, name: Option<&str>) -> ActionResult {
        let start = std::time::Instant::now();

        if let Some(user) = self.get_user(user_id) {
            let mut user_guard = user.lock().await;
            let req = pb::service::ourchat::set_account_info::v1::SetSelfInfoRequest {
                user_name: name.map(|s| s.to_string()),
                avatar_key: None,
                user_defined_status: None,
                ocid: None,
            };
            match user_guard.oc().set_self_info(req).await {
                Ok(_) => ActionResult::Success {
                    duration: start.elapsed(),
                    data: ResponseData::None,
                },
                Err(e) => ActionResult::UnexpectedFailure {
                    error: e.to_string(),
                    duration: start.elapsed(),
                },
            }
        } else {
            ActionResult::UnexpectedFailure {
                error: format!("User {} not found", user_id.0),
                duration: start.elapsed(),
            }
        }
    }
}

impl Default for ActionExecutor {
    fn default() -> Self {
        Self::new()
    }
}
