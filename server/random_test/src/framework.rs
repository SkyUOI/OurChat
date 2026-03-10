use super::actions::{ActionExecutor, ActionResult, RandomAction, ResponseData};
use super::config::RandomTestConfig;
use super::generator::ActionGenerator;
use super::metrics::MetricsCollector;
use super::state::{MessageRecord, TestState, UserState};
use super::validators::StateValidator;
use base::constants::ID;
use client::ClientCore;
use client::oc_helper::user::TestUser;
use rand::RngExt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{error, info, warn};

/// Core random test engine
pub struct RandomTestEngine {
    /// Configuration
    config: RandomTestConfig,

    /// Client core for gRPC connections
    client_core: ClientCore,

    /// Test state
    state: Arc<TestState>,

    /// Metrics collector
    metrics: Arc<MetricsCollector>,

    /// Action generator
    generator: ActionGenerator,

    /// Action executor
    executor: Arc<ActionExecutor>,

    /// Shutdown signal
    shutdown: Arc<AtomicBool>,

    /// Test users
    users: Vec<Arc<Mutex<TestUser>>>,
}

impl RandomTestEngine {
    /// Create a new random test engine
    pub fn new(config: RandomTestConfig, client_core: ClientCore) -> anyhow::Result<Self> {
        // Create action generator from embedded weights
        let weights = config.action_weights.clone();
        let generator = if config.seed > 0 {
            ActionGenerator::from_seed(weights, config.seed)
        } else {
            ActionGenerator::new(weights)
        };

        Ok(Self {
            config,
            client_core,
            state: Arc::new(TestState::new()),
            metrics: Arc::new(MetricsCollector::new()),
            generator,
            executor: Arc::new(ActionExecutor::new()),
            shutdown: Arc::new(AtomicBool::new(false)),
            users: Vec::new(),
        })
    }

    /// Run the random test
    pub async fn run(&mut self) -> anyhow::Result<()> {
        info!("═══════════════════════════════════════════════════════════════");
        info!("🎲 OurChat Random Test Starting");
        info!("═══════════════════════════════════════════════════════════════");
        info!("Configuration:");
        info!("  Users: {}", self.config.num_users);
        info!("  Duration: {:?}", self.config.running_duration);
        info!("  Action Rate: {}/sec", self.config.actions_per_second);
        info!("  Concurrency: {}", self.config.concurrency);
        info!("  Seed: {}", self.config.seed);

        // TODO: Setup shutdown handler

        // Initialize users
        info!("Creating {} test users...", self.config.num_users);
        self.initialize_users().await?;

        // Start action executor tasks
        info!("Starting action executors...");
        self.run_action_loop().await?;

        // Generate final report
        self.print_final_report().await;

        Ok(())
    }

    /// Initialize test users
    async fn initialize_users(&mut self) -> anyhow::Result<()> {
        for i in 0..self.config.num_users {
            let mut user = TestUser::random_readable(&self.client_core).await;
            user.register()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to register user {}: {}", i, e))?;

            // Add to state - use the user's fields directly before wrapping
            let user_state = UserState {
                id: user.id,
                _ocid: user.ocid.clone(),
                _name: user.name.clone(),
                _email: user.email.clone(),
                _is_online: true,
                _created_at: Instant::now(),
                _last_activity: Instant::now(),
            };
            self.state.add_user(user_state).await?;

            // Wrap in Arc<Mutex> and store
            let user_arc = Arc::new(Mutex::new(user));
            self.users.push(user_arc.clone());
            self.executor.register_user(user_arc).await;

            if (i + 1) % 10 == 0 {
                info!("Created {}/{} users", i + 1, self.config.num_users);
            }
        }

        info!("All {} users created successfully", self.config.num_users);
        Ok(())
    }

    /// Run the main action loop
    async fn run_action_loop(&mut self) -> anyhow::Result<()> {
        let start_time = Instant::now();
        let test_duration = self.config.running_duration;
        let action_interval = self.config.action_interval();
        let validation_interval = self.config.validation_interval;
        let metrics_interval = self.config.metrics_interval;

        let mut action_count = 0;
        let mut last_validation = start_time;
        let mut last_metrics = start_time;

        info!("Starting action execution loop...");

        while !self.shutdown.load(Ordering::SeqCst) && start_time.elapsed() < test_duration {
            // Generate action
            if let Some(action) = self.generator.generate(&self.state) {
                // Execute action
                let result = self.executor.execute(&action).await;

                // Record metrics
                self.metrics.record_action(&action, &result);

                // Update state based on result
                self.update_state(&action, &result).await;

                action_count += 1;

                // Check for unexpected errors
                if matches!(result, ActionResult::UnexpectedFailure { .. }) {
                    let error_count = self.metrics.unexpected_error_count.load(Ordering::SeqCst);
                    if error_count >= self.config.max_errors && self.config.max_errors > 0 {
                        warn!(
                            "Maximum error count ({}) reached, stopping test",
                            self.config.max_errors
                        );
                        self.shutdown.store(true, Ordering::SeqCst);
                        break;
                    }
                }
            }

            // Periodic validation
            if last_validation.elapsed() >= validation_interval {
                self.validate_state().await;
                last_validation = Instant::now();
            }

            // Periodic metrics reporting
            if last_metrics.elapsed() >= metrics_interval {
                let stats = self.state.get_stats();
                self.metrics.record_checkpoint(stats.clone());
                self.metrics.print_progress(&stats);
                last_metrics = Instant::now();
            }

            // Sleep for action interval
            sleep(action_interval).await;
        }

        info!("Action loop completed. Total actions: {}", action_count);
        Ok(())
    }

    /// Update state based on action result
    async fn update_state(&self, action: &RandomAction, result: &ActionResult) {
        // Only update state on success
        if !result.is_success() {
            return;
        }

        match action {
            // Message actions
            RandomAction::SendMessage {
                session_id,
                user_id,
                ..
            } => {
                let msg = MessageRecord {
                    _msg_id: rand::rng().random_range(1_u64..1000000),
                    _sender_id: *user_id,
                    session_id: *session_id,
                    _timestamp: chrono::Utc::now().timestamp(),
                };
                let _ = self.state.add_message(msg).await;

                // Update message count in session
                if let Some(mut session) = self.state.sessions.get_mut(session_id) {
                    session.message_count += 1;
                }
            }

            // Session management actions
            RandomAction::CreateSession {
                name,
                e2ee_enabled,
                initial_members,
            } => {
                // Extract session_id from response data
                if let ResponseData::SessionCreated(session_id) = result.get_data() {
                    let owner_id = *initial_members.first().unwrap_or(&ID(0));

                    let session = super::state::SessionState {
                        id: session_id,
                        _name: name.clone(),
                        owner_id,
                        member_ids: initial_members.clone(),
                        _e2ee_enabled: *e2ee_enabled,
                        _created_at: std::time::Instant::now(),
                        message_count: 0,
                    };
                    let _ = self.state.add_session(session).await;
                }
            }

            RandomAction::DeleteSession { session_id } => {
                let _ = self.state.remove_session(*session_id).await;
            }

            RandomAction::JoinSession {
                session_id,
                user_id,
            } => {
                // Add user to session
                if let Some(mut session) = self.state.sessions.get_mut(session_id)
                    && !session.member_ids.contains(user_id)
                {
                    session.member_ids.push(*user_id);
                }
                // Update user-to-sessions mapping
                let mut entry = self.state.user_sessions.entry(*user_id).or_default();
                if !entry.contains(session_id) {
                    entry.push(*session_id);
                }
            }

            RandomAction::LeaveSession {
                session_id,
                user_id,
            } => {
                // Remove user from session
                if let Some(mut session) = self.state.sessions.get_mut(session_id) {
                    session.member_ids.retain(|id| *id != *user_id);
                }
                // Update user-to-sessions mapping
                if let Some(mut entry) = self.state.user_sessions.get_mut(user_id) {
                    entry.retain(|id| *id != *session_id);
                }
            }

            RandomAction::InviteToSession {
                session_id,
                inviter_id,
                invitee_id,
            } => {
                // Track the pending session invitation
                let _ = self
                    .state
                    .add_pending_session_invitation(*inviter_id, *invitee_id, *session_id)
                    .await;
            }

            RandomAction::AcceptSessionInvitation {
                session_id,
                inviter_id,
                user_id,
                accepted: true,
            } => {
                // Add user to session when invitation is accepted
                if let Some(mut session) = self.state.sessions.get_mut(session_id)
                    && !session.member_ids.contains(user_id)
                {
                    session.member_ids.push(*user_id);
                }
                // Update user-to-sessions mapping
                let mut entry = self.state.user_sessions.entry(*user_id).or_default();
                if !entry.contains(session_id) {
                    entry.push(*session_id);
                }
                // Remove the pending invitation
                let _ = self
                    .state
                    .remove_pending_session_invitation(*inviter_id, *user_id, *session_id)
                    .await;
            }

            RandomAction::AcceptSessionInvitation {
                session_id,
                inviter_id,
                user_id,
                accepted: false,
            } => {
                // Invitation rejected - remove the pending invitation
                let _ = self
                    .state
                    .remove_pending_session_invitation(*inviter_id, *user_id, *session_id)
                    .await;
            }

            // Session moderation actions
            RandomAction::KickUser {
                session_id,
                user_id: target_id,
                ..
            } => {
                // Remove user from session
                if let Some(mut session) = self.state.sessions.get_mut(session_id) {
                    session.member_ids.retain(|id| *id != *target_id);
                }
                // Update user-to-sessions mapping
                if let Some(mut entry) = self.state.user_sessions.get_mut(target_id) {
                    entry.retain(|id| *id != *session_id);
                }
            }

            RandomAction::BanUser {
                session_id,
                user_id: target_id,
                duration,
                ..
            } => {
                let restriction = super::state::RestrictionState {
                    is_banned: true,
                    is_muted: false,
                    ban_expiry: duration.and_then(|d| std::time::Instant::now().checked_add(d)),
                    mute_expiry: None,
                };
                self.state
                    .restrictions
                    .insert((*target_id, *session_id), restriction);
            }

            RandomAction::UnbanUser {
                session_id,
                user_id: target_id,
                ..
            } => {
                if let Some(mut restriction) =
                    self.state.restrictions.get_mut(&(*target_id, *session_id))
                {
                    restriction.is_banned = false;
                    restriction.ban_expiry = None;
                }
            }

            RandomAction::MuteUser {
                session_id,
                user_id: target_id,
                duration,
                ..
            } => {
                let restriction = super::state::RestrictionState {
                    is_banned: false,
                    is_muted: true,
                    ban_expiry: None,
                    mute_expiry: duration.and_then(|d| std::time::Instant::now().checked_add(d)),
                };
                self.state
                    .restrictions
                    .insert((*target_id, *session_id), restriction);
            }

            RandomAction::UnmuteUser {
                session_id,
                user_id: target_id,
                ..
            } => {
                if let Some(mut restriction) =
                    self.state.restrictions.get_mut(&(*target_id, *session_id))
                {
                    restriction.is_muted = false;
                    restriction.mute_expiry = None;
                }
            }

            // Friend management actions
            RandomAction::AddFriend {
                user_id, friend_id, ..
            } => {
                // Add as pending invitation
                let _ = self
                    .state
                    .add_pending_invitation(*user_id, *friend_id)
                    .await;
            }

            RandomAction::AcceptFriendInvitation {
                user_id,
                friend_id,
                accepted: true,
            } => {
                // Convert pending invitation to friendship
                let _ = self.state.add_friendship(*user_id, *friend_id).await;
            }

            RandomAction::AcceptFriendInvitation {
                accepted: false, ..
            } => {
                // Invitation rejected - remove from pending
                // (Handled by add_friendship which removes from pending)
            }

            RandomAction::DeleteFriend { user_id, friend_id } => {
                let _ = self.state.remove_friendship(*user_id, *friend_id).await;
            }

            // File operations - no state tracking needed currently
            RandomAction::UploadFile { .. } | RandomAction::DownloadFile { .. } => {}

            // Info query operations - no state change
            RandomAction::FetchMessages { .. }
            | RandomAction::RecallMessage { .. }
            | RandomAction::GetSessionInfo { .. }
            | RandomAction::GetAccountInfo { .. }
            | RandomAction::SetSessionInfo { .. }
            | RandomAction::SetAccountInfo { .. } => {}
        }
    }

    /// Validate state consistency
    async fn validate_state(&self) {
        let result = StateValidator::validate_all(&self.state);

        if !result.is_valid {
            warn!(
                "State validation failed: {} issues detected",
                result.issues.len()
            );

            for issue in &result.issues {
                match issue.severity {
                    super::validators::IssueSeverity::Critical => {
                        error!("CRITICAL: {}", issue.message);
                        self.metrics.record_anomaly(
                            super::metrics::AnomalySeverity::Critical,
                            issue.message.clone(),
                        );
                    }
                    super::validators::IssueSeverity::Error => {
                        error!("ERROR: {}", issue.message);
                        self.metrics.record_anomaly(
                            super::metrics::AnomalySeverity::Error,
                            issue.message.clone(),
                        );
                    }
                    super::validators::IssueSeverity::Warning => {
                        warn!("WARNING: {}", issue.message);
                        self.metrics.record_anomaly(
                            super::metrics::AnomalySeverity::Warning,
                            issue.message.clone(),
                        );
                    }
                }
            }
        }
    }

    /// Print final report
    async fn print_final_report(&self) {
        info!("");
        info!("═══════════════════════════════════════════════════════════════");
        info!("                    Test Completed                              ");
        info!("═══════════════════════════════════════════════════════════════");

        let report = self.metrics.generate_report();
        println!("\n{}", report);

        // Save report to JSON
        if let Err(e) = self.save_report_to_json(&report) {
            warn!("Failed to save report to JSON: {}", e);
        }
    }

    /// Save report to JSON file
    fn save_report_to_json(&self, report: &super::metrics::RandomTestReport) -> anyhow::Result<()> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("random_test_report_{}.json", timestamp);

        let json = serde_json::to_string_pretty(report)?;
        std::fs::write(&filename, json)?;

        info!("Report saved to: {}", filename);
        Ok(())
    }

    /// Cleanup resources
    pub async fn cleanup(self) {
        info!("Cleaning up...");

        // Unregister all users
        for user in &self.users {
            let mut user_guard = user.lock().await;
            if let Err(e) = user_guard.unregister().await {
                warn!("Failed to unregister user: {}", e);
            }
        }

        info!("Cleanup complete");
    }
}
