use base::constants::{ID, OCID, SessionID};
use dashmap::DashMap;
use rand::RngExt;
use std::time::Instant;

/// State for tracking messages
#[derive(Clone, Debug)]
pub struct MessageRecord {
    pub _msg_id: u64,
    pub _sender_id: ID,
    pub session_id: SessionID,
    pub _timestamp: i64,
}

/// State for tracking users
#[derive(Clone, Debug)]
pub struct UserState {
    pub id: ID,
    pub _ocid: OCID,
    pub _name: String,
    pub _email: String,
    pub _is_online: bool,
    pub _created_at: Instant,
    pub _last_activity: Instant,
}

/// State for tracking sessions
#[derive(Clone, Debug)]
pub struct SessionState {
    pub id: SessionID,
    pub _name: String,
    pub owner_id: ID,
    pub member_ids: Vec<ID>,
    pub _e2ee_enabled: bool,
    pub _created_at: Instant,
    pub message_count: u64,
}

/// State for tracking restrictions
#[derive(Clone, Debug)]
pub struct RestrictionState {
    pub is_banned: bool,
    pub is_muted: bool,
    pub ban_expiry: Option<Instant>,
    pub mute_expiry: Option<Instant>,
}

/// Thread-safe test state
pub struct TestState {
    /// All registered users
    pub users: DashMap<ID, UserState>,
    /// All active sessions
    pub sessions: DashMap<SessionID, SessionState>,
    /// Friendship relationships (bidirectional mapping)
    pub friendships: DashMap<ID, Vec<ID>>,
    /// Pending friend invitations (target -> list of inviters)
    pub pending_invitations: DashMap<ID, Vec<ID>>,
    /// Pending session invitations (target -> list of (inviter, session_id))
    pub pending_session_invitations: DashMap<ID, Vec<(ID, SessionID)>>,
    /// User-to-sessions mapping
    pub user_sessions: DashMap<ID, Vec<SessionID>>,
    /// Ban/mute status cache
    pub restrictions: DashMap<(ID, SessionID), RestrictionState>,
    /// Message history for validation
    pub message_log: DashMap<SessionID, Vec<MessageRecord>>,
}

impl TestState {
    pub fn new() -> Self {
        Self {
            users: DashMap::new(),
            sessions: DashMap::new(),
            friendships: DashMap::new(),
            pending_invitations: DashMap::new(),
            pending_session_invitations: DashMap::new(),
            user_sessions: DashMap::new(),
            restrictions: DashMap::new(),
            message_log: DashMap::new(),
        }
    }

    /// Add a user to the state
    pub async fn add_user(&self, user: UserState) -> anyhow::Result<()> {
        let user_id = user.id;
        self.users.insert(user_id, user);
        // Initialize empty friendship list
        self.friendships.insert(user_id, Vec::new());
        Ok(())
    }

    /// Add a session to the state
    pub async fn add_session(&self, session: SessionState) -> anyhow::Result<()> {
        // Add session
        self.sessions.insert(session.id, session.clone());

        // Update user-to-sessions mapping
        for member_id in &session.member_ids {
            let mut entry = self.user_sessions.entry(*member_id).or_default();
            if !entry.contains(&session.id) {
                entry.push(session.id);
            }
        }

        Ok(())
    }

    /// Remove a session from the state
    pub async fn remove_session(&self, session_id: SessionID) -> anyhow::Result<()> {
        if let Some((_, session)) = self.sessions.remove(&session_id) {
            // Update user-to-sessions mapping
            for member_id in &session.member_ids {
                if let Some(mut entry) = self.user_sessions.get_mut(member_id) {
                    entry.retain(|id| *id != session_id);
                }
            }
        }
        Ok(())
    }

    /// Add a friendship (bidirectional)
    pub async fn add_friendship(&self, user1: ID, user2: ID) -> anyhow::Result<()> {
        let mut friends1 = self.friendships.entry(user1).or_default();
        if !friends1.contains(&user2) {
            friends1.push(user2);
        }

        let mut friends2 = self.friendships.entry(user2).or_default();
        if !friends2.contains(&user1) {
            friends2.push(user1);
        }

        // Remove from pending invitations
        if let Some(mut pending) = self.pending_invitations.get_mut(&user2) {
            pending.retain(|id| *id != user1);
        }

        Ok(())
    }

    /// Remove a friendship (bidirectional)
    pub async fn remove_friendship(&self, user1: ID, user2: ID) -> anyhow::Result<()> {
        if let Some(mut friends1) = self.friendships.get_mut(&user1) {
            friends1.retain(|id| *id != user2);
        }

        if let Some(mut friends2) = self.friendships.get_mut(&user2) {
            friends2.retain(|id| *id != user1);
        }

        Ok(())
    }

    /// Add a pending friend invitation
    pub async fn add_pending_invitation(&self, from: ID, to: ID) -> anyhow::Result<()> {
        let mut pending = self.pending_invitations.entry(to).or_default();
        if !pending.contains(&from) {
            pending.push(from);
        }
        Ok(())
    }

    /// Add a pending session invitation
    pub async fn add_pending_session_invitation(
        &self,
        from: ID,
        to: ID,
        session_id: SessionID,
    ) -> anyhow::Result<()> {
        let mut pending = self.pending_session_invitations.entry(to).or_default();
        // Check if this invitation already exists
        if !pending
            .iter()
            .any(|(inviter, sid)| *inviter == from && *sid == session_id)
        {
            pending.push((from, session_id));
        }
        Ok(())
    }

    /// Remove a pending session invitation
    pub async fn remove_pending_session_invitation(
        &self,
        from: ID,
        to: ID,
        session_id: SessionID,
    ) -> anyhow::Result<()> {
        if let Some(mut pending) = self.pending_session_invitations.get_mut(&to) {
            pending.retain(|(inviter, sid)| !(*inviter == from && *sid == session_id));
        }
        Ok(())
    }

    /// Get a random pending session invitation for the given user
    pub fn get_random_pending_session_invitation(
        &self,
        user_id: ID,
    ) -> Option<(ID, ID, SessionID)> {
        if let Some(pending) = self.pending_session_invitations.get(&user_id) {
            let pending = pending.clone();
            if pending.is_empty() {
                None
            } else {
                let mut rng = rand::rng();
                let (inviter, session_id) = pending[rng.random_range(0..pending.len())];
                Some((user_id, inviter, session_id))
            }
        } else {
            None
        }
    }

    /// Add a message to the log
    pub async fn add_message(&self, msg: MessageRecord) -> anyhow::Result<()> {
        let mut log = self.message_log.entry(msg.session_id).or_default();
        log.push(msg);
        Ok(())
    }

    /// Get a random user ID
    pub fn get_random_user(&self) -> Option<ID> {
        let users: Vec<_> = self.users.iter().map(|entry| *entry.key()).collect();
        if users.is_empty() {
            None
        } else {
            let mut rng = rand::rng();
            Some(users[rng.random_range(0..users.len())])
        }
    }

    /// Get a random session ID
    pub fn get_random_session(&self) -> Option<SessionID> {
        let sessions: Vec<_> = self.sessions.iter().map(|entry| *entry.key()).collect();
        if sessions.is_empty() {
            None
        } else {
            let mut rng = rand::rng();
            Some(sessions[rng.random_range(0..sessions.len())])
        }
    }

    /// Get a random session that the user is a member of
    pub fn get_random_session_for_user(&self, user_id: ID) -> Option<SessionID> {
        if let Some(sessions) = self.user_sessions.get(&user_id) {
            let sessions = sessions.clone();
            if sessions.is_empty() {
                None
            } else {
                let mut rng = rand::rng();
                Some(sessions[rng.random_range(0..sessions.len())])
            }
        } else {
            None
        }
    }

    /// Get a random user that is not friends with the given user
    pub fn get_random_non_friend(&self, user_id: ID) -> Option<ID> {
        let friends = self.friendships.get(&user_id)?.clone();
        let users: Vec<_> = self
            .users
            .iter()
            .map(|entry| *entry.key())
            .filter(|id| *id != user_id && !friends.contains(id))
            .collect();

        if users.is_empty() {
            None
        } else {
            let mut rng = rand::rng();
            Some(users[rng.random_range(0..users.len())])
        }
    }

    /// Get a random user that is friends with the given user
    pub fn get_random_friend(&self, user_id: ID) -> Option<ID> {
        let friends = self.friendships.get(&user_id)?.clone();
        if friends.is_empty() {
            None
        } else {
            let mut rng = rand::rng();
            Some(friends[rng.random_range(0..friends.len())])
        }
    }

    /// Check if there's a pending invitation
    pub fn has_pending_invitation(&self, from: ID, to: ID) -> bool {
        if let Some(pending) = self.pending_invitations.get(&to) {
            pending.contains(&from)
        } else {
            false
        }
    }

    /// Get session members
    pub fn get_session_members(&self, session_id: SessionID) -> Option<Vec<ID>> {
        self.sessions.get(&session_id).map(|s| s.member_ids.clone())
    }

    /// Check if user is in session
    pub fn is_user_in_session(&self, user_id: ID, session_id: SessionID) -> bool {
        if let Some(session) = self.sessions.get(&session_id) {
            session.member_ids.contains(&user_id)
        } else {
            false
        }
    }

    /// Get current state statistics
    pub fn get_stats(&self) -> StateStats {
        StateStats {
            user_count: self.users.len(),
            session_count: self.sessions.len(),
            friendship_count: self
                .friendships
                .iter()
                .map(|entry| entry.value().len())
                .sum::<usize>()
                / 2,
            message_count: self
                .message_log
                .iter()
                .map(|entry| entry.value().len())
                .sum(),
        }
    }
}

/// State statistics snapshot
#[derive(Clone, Debug)]
pub struct StateStats {
    pub user_count: usize,
    pub session_count: usize,
    pub friendship_count: usize,
    pub message_count: usize,
}
