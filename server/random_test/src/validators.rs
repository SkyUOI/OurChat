use super::state::TestState;

/// Validation result
#[derive(Clone, Debug)]
pub struct ValidationResult {
    pub issues: Vec<ValidationIssue>,
    pub is_valid: bool,
}

/// Validation issue
#[derive(Clone, Debug)]
pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IssueSeverity {
    Warning,
    Error,
    Critical,
}

/// State validator for consistency checks
pub struct StateValidator;

impl StateValidator {
    /// Run all validation checks
    pub fn validate_all(state: &TestState) -> ValidationResult {
        let mut issues = Vec::new();

        // Run all validation checks
        issues.extend(Self::check_referential_integrity(state));
        issues.extend(Self::check_friendship_symmetry(state));
        issues.extend(Self::check_session_ownership(state));
        issues.extend(Self::check_orphaned_state(state));

        let is_valid = issues
            .iter()
            .all(|issue| issue.severity != IssueSeverity::Critical);

        ValidationResult { issues, is_valid }
    }

    /// Check referential integrity: all session members should exist as users
    fn check_referential_integrity(state: &TestState) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for session_entry in state.sessions.iter() {
            let session = session_entry.value();
            for member_id in &session.member_ids {
                if !state.users.contains_key(member_id) {
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Critical,
                        message: format!(
                            "Session {} contains non-existent member {}",
                            session.id, member_id.0
                        ),
                    });
                }
            }
        }

        // Check friendships refer to existing users
        for friendship_entry in state.friendships.iter() {
            let user_id = *friendship_entry.key();
            if !state.users.contains_key(&user_id) {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Critical,
                    message: format!("Friendship list exists for non-existent user {}", user_id.0),
                });
            }

            for friend_id in friendship_entry.value() {
                if !state.users.contains_key(friend_id) {
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Critical,
                        message: format!(
                            "User {} has friendship with non-existent user {}",
                            user_id.0, friend_id.0
                        ),
                    });
                }
            }
        }

        issues
    }

    /// Check friendship symmetry: if A is friends with B, B should be friends with A
    fn check_friendship_symmetry(state: &TestState) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for friendship_entry in state.friendships.iter() {
            let user_id = *friendship_entry.key();
            let friends = friendship_entry.value().clone();

            for friend_id in friends {
                // Check if the friendship is bidirectional
                if let Some(friend_friends) = state.friendships.get(&friend_id) {
                    if !friend_friends.contains(&user_id) {
                        issues.push(ValidationIssue {
                            severity: IssueSeverity::Error,
                            message: format!(
                                "Friendship asymmetry: {} is friends with {}, but {} is not friends with {}",
                                user_id.0, friend_id.0, friend_id.0, user_id.0
                            ),
                        });
                    }
                } else {
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Error,
                        message: format!(
                            "User {} has friend {} who has no friendship list",
                            user_id.0, friend_id.0
                        ),
                    });
                }
            }
        }

        issues
    }

    /// Check session ownership: session owners should be members of their sessions
    fn check_session_ownership(state: &TestState) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for session_entry in state.sessions.iter() {
            let session = session_entry.value();

            if !session.member_ids.contains(&session.owner_id) {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Error,
                    message: format!(
                        "Session {} owner {} is not a member of the session",
                        session.id, session.owner_id.0
                    ),
                });
            }
        }

        issues
    }

    /// Check for orphaned state (dangling references)
    fn check_orphaned_state(state: &TestState) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Check user_sessions mapping
        for user_session_entry in state.user_sessions.iter() {
            let user_id = *user_session_entry.key();
            if !state.users.contains_key(&user_id) {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Warning,
                    message: format!(
                        "user_sessions contains entry for non-existent user {}",
                        user_id.0
                    ),
                });
            }

            for session_id in user_session_entry.value() {
                if !state.sessions.contains_key(session_id) {
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Warning,
                        message: format!(
                            "User {} has reference to non-existent session {}",
                            user_id.0, session_id.0
                        ),
                    });
                }
            }
        }

        // Check restrictions mapping
        for restriction_entry in state.restrictions.iter() {
            let (user_id, session_id) = *restriction_entry.key();

            if !state.users.contains_key(&user_id) {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Warning,
                    message: format!(
                        "restrictions contains entry for non-existent user {}",
                        user_id.0
                    ),
                });
            }

            if !state.sessions.contains_key(&session_id) {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Warning,
                    message: format!(
                        "restrictions contains entry for non-existent session {}",
                        session_id.0
                    ),
                });
            }
        }

        // Check message_log
        for message_log_entry in state.message_log.iter() {
            let session_id = *message_log_entry.key();

            if !state.sessions.contains_key(&session_id) {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Warning,
                    message: format!(
                        "message_log contains entry for non-existent session {}",
                        session_id.0
                    ),
                });
            }
        }

        // Check pending_session_invitations
        for pending_inv_entry in state.pending_session_invitations.iter() {
            let user_id = *pending_inv_entry.key();
            if !state.users.contains_key(&user_id) {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Warning,
                    message: format!(
                        "pending_session_invitations contains entry for non-existent user {}",
                        user_id.0
                    ),
                });
            }

            for (inviter_id, session_id) in pending_inv_entry.value() {
                if !state.users.contains_key(inviter_id) {
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Warning,
                        message: format!(
                            "pending_session_invitations has invitation from non-existent user {}",
                            inviter_id.0
                        ),
                    });
                }
                if !state.sessions.contains_key(session_id) {
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Warning,
                        message: format!(
                            "pending_session_invitations has invitation to non-existent session {}",
                            session_id.0
                        ),
                    });
                }
            }
        }

        issues
    }
}
