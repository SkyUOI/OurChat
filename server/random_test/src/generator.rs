use super::actions::RandomAction;
use super::config::ActionWeights;
use super::state::TestState;
use base::constants::ID;
use fake::Fake;
use fake::faker::lorem::en::Sentence;
use fake::faker::name::en::FirstName;
use fake::faker::name::en::LastName;
use rand::RngExt;
use rand::SeedableRng;
use rand::rngs::StdRng;

/// Random action generator with weighted probabilities
pub struct ActionGenerator {
    weights: ActionWeights,
    rng: StdRng,
}

impl ActionGenerator {
    pub fn new(weights: ActionWeights) -> Self {
        // Use a random seed when not specified
        let seed = rand::random::<u64>();
        Self {
            weights,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    pub fn from_seed(weights: ActionWeights, seed: u64) -> Self {
        Self {
            weights,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// Choose a random element from a slice
    fn choose<'a, T>(rng: &mut StdRng, slice: &'a [T]) -> Option<&'a T> {
        if slice.is_empty() {
            None
        } else {
            let index = rng.random_range(0..slice.len());
            Some(&slice[index])
        }
    }

    /// Generate a random action based on current state and weights
    pub fn generate(&mut self, state: &TestState) -> Option<RandomAction> {
        // Calculate total weight
        let total_weight = self.calculate_total_weight(state);

        // Generate random value
        let mut random_value = self.rng.random_range(0.0..total_weight);

        // Select action based on weights
        // This is a simplified version - in production you'd use a more efficient algorithm

        // Try messaging actions first
        if let Some(user_id) = state.get_random_user() {
            // Try send message (most common)
            if let Some(session_id) = state.get_random_session_for_user(user_id)
                && self.random_select(&mut random_value, self.weights.send_message)
            {
                return Some(RandomAction::SendMessage {
                    session_id,
                    user_id,
                    content: Self::generate_message_content(&mut self.rng),
                    is_encrypted: self.rng.random_range(0..10) < 2, // 20% encrypted
                });
            }

            // Try recall message (less common)
            if self.random_select(&mut random_value, self.weights.recall_message)
                && let Some(session_id) = state.get_random_session_for_user(user_id)
            {
                return Some(RandomAction::RecallMessage {
                    session_id,
                    user_id,
                    message_id: self.rng.random_range(1..10000),
                });
            }

            // Try get session info
            if self.random_select(&mut random_value, self.weights.get_session_info)
                && let Some(session_id) = state.get_random_session()
            {
                return Some(RandomAction::GetSessionInfo {
                    user_id,
                    session_id,
                });
            }

            // Try get account info
            if self.random_select(&mut random_value, self.weights.get_account_info)
                && let Some(target_id) = state.get_random_user()
            {
                return Some(RandomAction::GetAccountInfo { user_id, target_id });
            }

            // Try add friend
            if self.random_select(&mut random_value, self.weights.add_friend)
                && let Some(friend_id) = state.get_random_non_friend(user_id)
            {
                return Some(RandomAction::AddFriend {
                    user_id,
                    friend_id,
                    message: if self.rng.random() {
                        Some(Self::generate_message_content(&mut self.rng))
                    } else {
                        None
                    },
                });
            }

            // Try accept friend invitation
            if self.random_select(&mut random_value, self.weights.accept_friend_invitation)
                && let Some(friend_id) = state.get_random_non_friend(user_id)
                && state.has_pending_invitation(friend_id, user_id)
            {
                return Some(RandomAction::AcceptFriendInvitation {
                    user_id,
                    friend_id,
                    accepted: self.rng.random(), // Randomly accept or reject
                });
            }

            // Try delete friend
            if self.random_select(&mut random_value, self.weights.delete_friend)
                && let Some(friend_id) = state.get_random_friend(user_id)
            {
                return Some(RandomAction::DeleteFriend { user_id, friend_id });
            }

            // Try create session
            if self.random_select(&mut random_value, self.weights.create_session) {
                let members = self.generate_random_members(state, user_id, 2..5);
                if members.len() >= 2 {
                    return Some(RandomAction::CreateSession {
                        name: Self::generate_session_name(&mut self.rng),
                        e2ee_enabled: self.rng.random_range(0..10) < 3, // 30% E2EE
                        initial_members: members,
                    });
                }
            }

            // Try delete session
            if self.random_select(&mut random_value, self.weights.delete_session)
                && let Some(session_id) = state.get_random_session()
            {
                return Some(RandomAction::DeleteSession { session_id });
            }

            // Try join session
            if self.random_select(&mut random_value, self.weights.join_session)
                && let Some(session_id) = state.get_random_session()
                && !state.is_user_in_session(user_id, session_id)
            {
                return Some(RandomAction::JoinSession {
                    session_id,
                    user_id,
                });
            }

            // Try leave session
            if self.random_select(&mut random_value, self.weights.leave_session)
                && let Some(session_id) = state.get_random_session_for_user(user_id)
            {
                return Some(RandomAction::LeaveSession {
                    session_id,
                    user_id,
                });
            }

            // Try invite to session
            if self.random_select(&mut random_value, self.weights.invite_to_session)
                && let Some(session_id) = state.get_random_session_for_user(user_id)
                && let Some(invitee_id) = state.get_random_non_friend(user_id)
                && !state.is_user_in_session(invitee_id, session_id)
            {
                return Some(RandomAction::InviteToSession {
                    session_id,
                    inviter_id: user_id,
                    invitee_id,
                });
            }

            // Try accept session invitation
            if self.random_select(&mut random_value, self.weights.accept_session_invitation)
                && let Some((user_id, inviter_id, session_id)) =
                    state.get_random_pending_session_invitation(user_id)
            {
                return Some(RandomAction::AcceptSessionInvitation {
                    session_id,
                    inviter_id,
                    user_id,
                    accepted: self.rng.random(), // Randomly accept or reject
                });
            }

            // Try kick user
            if self.random_select(&mut random_value, self.weights.kick_user)
                && let Some(session_id) = state.get_random_session_for_user(user_id)
                && let Some(members) = state.get_session_members(session_id)
                && members.len() > 1
                && let Some(&target_id) = Self::choose(&mut self.rng, members.as_slice())
                && target_id != user_id
            {
                return Some(RandomAction::KickUser {
                    session_id,
                    moderator_id: user_id,
                    user_id: target_id,
                });
            }

            // Try ban user
            if self.random_select(&mut random_value, self.weights.ban_user)
                && let Some(session_id) = state.get_random_session_for_user(user_id)
                && let Some(members) = state.get_session_members(session_id)
                && let Some(&target_id) = Self::choose(&mut self.rng, members.as_slice())
                && target_id != user_id
            {
                return Some(RandomAction::BanUser {
                    session_id,
                    moderator_id: user_id,
                    user_id: target_id,
                    duration: if self.rng.random() {
                        Some(std::time::Duration::from_secs(
                            self.rng.random_range(60..3600),
                        ))
                    } else {
                        None
                    },
                });
            }

            // Try unban user
            if self.random_select(&mut random_value, self.weights.unban_user)
                && let Some(session_id) = state.get_random_session_for_user(user_id)
                && let Some(members) = state.get_session_members(session_id)
                && let Some(&target_id) = Self::choose(&mut self.rng, members.as_slice())
            {
                return Some(RandomAction::UnbanUser {
                    session_id,
                    moderator_id: user_id,
                    user_id: target_id,
                });
            }

            // Try mute user
            if self.random_select(&mut random_value, self.weights.mute_user)
                && let Some(session_id) = state.get_random_session_for_user(user_id)
                && let Some(members) = state.get_session_members(session_id)
                && let Some(&target_id) = Self::choose(&mut self.rng, members.as_slice())
                && target_id != user_id
            {
                return Some(RandomAction::MuteUser {
                    session_id,
                    moderator_id: user_id,
                    user_id: target_id,
                    duration: if self.rng.random() {
                        Some(std::time::Duration::from_secs(
                            self.rng.random_range(60..1800),
                        ))
                    } else {
                        None
                    },
                });
            }

            // Try unmute user
            if self.random_select(&mut random_value, self.weights.unmute_user)
                && let Some(session_id) = state.get_random_session_for_user(user_id)
                && let Some(members) = state.get_session_members(session_id)
                && let Some(&target_id) = Self::choose(&mut self.rng, members.as_slice())
            {
                return Some(RandomAction::UnmuteUser {
                    session_id,
                    moderator_id: user_id,
                    user_id: target_id,
                });
            }

            // Try upload file
            if self.random_select(&mut random_value, self.weights.upload_file)
                && let Some(session_id) = state.get_random_session_for_user(user_id)
            {
                return Some(RandomAction::UploadFile {
                    user_id,
                    session_id: Some(session_id),
                    size: self.rng.random_range(1024..10 * 1024 * 1024), // 1KB to 10MB
                });
            }

            // Try download file
            if self.random_select(&mut random_value, self.weights.download_file) {
                return Some(RandomAction::DownloadFile {
                    user_id,
                    file_key: format!("file_{}", self.rng.random_range(0_u64..10000)),
                });
            }

            // Try set session info
            if self.random_select(&mut random_value, self.weights.set_session_info)
                && let Some(session_id) = state.get_random_session_for_user(user_id)
            {
                return Some(RandomAction::SetSessionInfo {
                    user_id,
                    session_id,
                    name: if self.rng.random() {
                        Some(Self::generate_session_name(&mut self.rng))
                    } else {
                        None
                    },
                });
            }

            // Try set account info
            if self.random_select(&mut random_value, self.weights.set_account_info) {
                return Some(RandomAction::SetAccountInfo {
                    user_id,
                    name: if self.rng.random() {
                        Some(Self::generate_user_name(&mut self.rng))
                    } else {
                        None
                    },
                });
            }
        }

        // Fallback: if no action could be generated, return None
        None
    }

    /// Helper for weighted random selection
    fn random_select(&self, value: &mut f64, weight: f64) -> bool {
        if *value < weight {
            *value = f64::MAX; // Prevent further selections
            true
        } else {
            *value -= weight;
            false
        }
    }

    /// Calculate total weight considering current state
    fn calculate_total_weight(&self, state: &TestState) -> f64 {
        let mut total = 0.0;

        // Always available actions
        total += self.weights.send_message;
        total += self.weights.get_account_info;

        // Conditionally available actions
        if !state.sessions.is_empty() {
            total += self.weights.get_session_info;
            total += self.weights.recall_message;
            total += self.weights.delete_session;
        }

        if !state.users.is_empty() {
            total += self.weights.add_friend;
            total += self.weights.delete_friend;
        }

        total += self.weights.create_session;
        total += self.weights.join_session;
        total += self.weights.leave_session;
        total += self.weights.invite_to_session;
        total += self.weights.kick_user;
        total += self.weights.ban_user;
        total += self.weights.unban_user;
        total += self.weights.mute_user;
        total += self.weights.unmute_user;
        total += self.weights.upload_file;
        total += self.weights.download_file;
        total += self.weights.set_session_info;
        total += self.weights.set_account_info;

        total
    }

    /// Generate random session members including the creator
    fn generate_random_members(
        &mut self,
        state: &TestState,
        creator_id: ID,
        range: std::ops::Range<usize>,
    ) -> Vec<ID> {
        let count = self.rng.random_range(range);
        let mut members = vec![creator_id];

        let all_users: Vec<_> = state.users.iter().map(|entry| *entry.key()).collect();
        let mut available: Vec<_> = all_users
            .into_iter()
            .filter(|id| *id != creator_id)
            .collect();

        for _ in 0..count.min(available.len()) {
            if let Some(&selected_id) = Self::choose(&mut self.rng, &available) {
                members.push(selected_id);
                available.retain(|id| *id != selected_id);
            }
        }

        members
    }

    /// Generate realistic message content using fake with seeded RNG
    fn generate_message_content(rng: &mut StdRng) -> String {
        const EMOJIS: &[&str] = &["😀", "😂", "😍", "🤔", "😎", "👍", "👎", "❤️", "🔥", "✨"];

        let message_type = rng.random_range(0u8..10);

        match message_type {
            0..=7 => {
                // Simple sentence (80%)
                Sentence(1..2).fake_with_rng::<String, _>(rng)
            }
            8 => {
                // Sentence with emoji (10%)
                format!(
                    "{} {}",
                    Sentence(1..2).fake_with_rng::<String, _>(rng),
                    Self::choose(rng, EMOJIS).unwrap()
                )
            }
            _ => {
                // Longer message (10%)
                format!(
                    "{} {}",
                    Sentence(1..2).fake_with_rng::<String, _>(rng),
                    Sentence(1..2).fake_with_rng::<String, _>(rng)
                )
            }
        }
    }

    /// Generate random session name using fake with seeded RNG
    fn generate_session_name(rng: &mut StdRng) -> String {
        const GROUP_TYPES: &[&str] = &[
            "Group", "Chat", "Team", "Squad", "Crew", "Club", "Party", "Hangout", "Room", "Space",
            "Lounge", "Hub", "Corner", "Spot", "Place", "Zone",
        ];

        let first_name: String = FirstName().fake_with_rng(rng);
        let group_index = rng.random_range(0..GROUP_TYPES.len());
        format!("{}'s {}", first_name, GROUP_TYPES[group_index])
    }

    /// Generate random user name using fake with seeded RNG
    fn generate_user_name(rng: &mut StdRng) -> String {
        format!(
            "{} {}",
            FirstName().fake_with_rng::<String, _>(rng),
            LastName().fake_with_rng::<String, _>(rng)
        )
    }
}
