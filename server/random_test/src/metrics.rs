use super::actions::{ActionResult, RandomAction};
use super::state::StateStats;
use comfy_table::Table;
use dashmap::DashMap;
use hdrhistogram::Histogram;
use std::io::Write;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// Action type for tracking statistics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub enum RandomActionType {
    SendMessage,
    RecallMessage,
    CreateSession,
    DeleteSession,
    JoinSession,
    LeaveSession,
    InviteToSession,
    AcceptSessionInvitation,
    KickUser,
    BanUser,
    UnbanUser,
    MuteUser,
    UnmuteUser,
    AddFriend,
    AcceptFriendInvitation,
    DeleteFriend,
    UploadFile,
    DownloadFile,
    GetSessionInfo,
    GetAccountInfo,
    SetSessionInfo,
    SetAccountInfo,
}

impl From<&RandomAction> for RandomActionType {
    fn from(action: &RandomAction) -> Self {
        match action {
            RandomAction::SendMessage { .. } => RandomActionType::SendMessage,
            RandomAction::RecallMessage { .. } => RandomActionType::RecallMessage,
            RandomAction::CreateSession { .. } => RandomActionType::CreateSession,
            RandomAction::DeleteSession { .. } => RandomActionType::DeleteSession,
            RandomAction::JoinSession { .. } => RandomActionType::JoinSession,
            RandomAction::LeaveSession { .. } => RandomActionType::LeaveSession,
            RandomAction::InviteToSession { .. } => RandomActionType::InviteToSession,
            RandomAction::AcceptSessionInvitation { .. } => {
                RandomActionType::AcceptSessionInvitation
            }
            RandomAction::KickUser { .. } => RandomActionType::KickUser,
            RandomAction::BanUser { .. } => RandomActionType::BanUser,
            RandomAction::UnbanUser { .. } => RandomActionType::UnbanUser,
            RandomAction::MuteUser { .. } => RandomActionType::MuteUser,
            RandomAction::UnmuteUser { .. } => RandomActionType::UnmuteUser,
            RandomAction::AddFriend { .. } => RandomActionType::AddFriend,
            RandomAction::AcceptFriendInvitation { .. } => RandomActionType::AcceptFriendInvitation,
            RandomAction::DeleteFriend { .. } => RandomActionType::DeleteFriend,
            RandomAction::UploadFile { .. } => RandomActionType::UploadFile,
            RandomAction::DownloadFile { .. } => RandomActionType::DownloadFile,
            RandomAction::GetSessionInfo { .. } => RandomActionType::GetSessionInfo,
            RandomAction::GetAccountInfo { .. } => RandomActionType::GetAccountInfo,
            RandomAction::SetSessionInfo { .. } => RandomActionType::SetSessionInfo,
            RandomAction::SetAccountInfo { .. } => RandomActionType::SetAccountInfo,
        }
    }
}

impl std::fmt::Display for RandomActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RandomActionType::SendMessage => write!(f, "send_msg"),
            RandomActionType::RecallMessage => write!(f, "recall_msg"),
            RandomActionType::CreateSession => write!(f, "create_session"),
            RandomActionType::DeleteSession => write!(f, "delete_session"),
            RandomActionType::JoinSession => write!(f, "join_session"),
            RandomActionType::LeaveSession => write!(f, "leave_session"),
            RandomActionType::InviteToSession => write!(f, "invite_to_session"),
            RandomActionType::AcceptSessionInvitation => write!(f, "accept_invite"),
            RandomActionType::KickUser => write!(f, "kick_user"),
            RandomActionType::BanUser => write!(f, "ban_user"),
            RandomActionType::UnbanUser => write!(f, "unban_user"),
            RandomActionType::MuteUser => write!(f, "mute_user"),
            RandomActionType::UnmuteUser => write!(f, "unmute_user"),
            RandomActionType::AddFriend => write!(f, "add_friend"),
            RandomActionType::AcceptFriendInvitation => write!(f, "accept_friend"),
            RandomActionType::DeleteFriend => write!(f, "delete_friend"),
            RandomActionType::UploadFile => write!(f, "upload_file"),
            RandomActionType::DownloadFile => write!(f, "download_file"),
            RandomActionType::GetSessionInfo => write!(f, "get_session_info"),
            RandomActionType::GetAccountInfo => write!(f, "get_account_info"),
            RandomActionType::SetSessionInfo => write!(f, "set_session_info"),
            RandomActionType::SetAccountInfo => write!(f, "set_account_info"),
        }
    }
}

/// Error record for tracking issues
#[derive(Clone, Debug, serde::Serialize)]
pub struct ErrorRecord {
    pub action_type: RandomActionType,
    pub error: String,
    pub user_id: Option<u64>,
}

/// Anomaly detected during testing
#[derive(Clone, Debug, serde::Serialize)]
pub struct Anomaly {
    pub severity: AnomalySeverity,
    pub message: String,
}

#[derive(Clone, Debug, serde::Serialize)]
pub enum AnomalySeverity {
    Warning,
    Error,
    Critical,
}

/// Metrics collector for tracking test statistics
pub struct MetricsCollector {
    // Action counters
    pub action_counts: DashMap<RandomActionType, usize>,
    pub success_counts: DashMap<RandomActionType, usize>,
    pub failure_counts: DashMap<RandomActionType, usize>,

    // Latency tracking
    latency_histograms: DashMap<RandomActionType, Histogram<u64>>,

    // Error tracking
    pub errors: Mutex<Vec<ErrorRecord>>,
    pub unexpected_error_count: AtomicUsize,

    // State tracking
    pub peak_users: AtomicUsize,
    pub peak_sessions: AtomicUsize,
    pub peak_friendships: AtomicUsize,
    pub total_messages: AtomicUsize,

    // Timeline tracking
    pub start_time: Instant,
    pub anomalies: Mutex<Vec<Anomaly>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            action_counts: DashMap::new(),
            success_counts: DashMap::new(),
            failure_counts: DashMap::new(),
            latency_histograms: DashMap::new(),
            errors: Mutex::new(Vec::new()),
            unexpected_error_count: AtomicUsize::new(0),
            peak_users: AtomicUsize::new(0),
            peak_sessions: AtomicUsize::new(0),
            peak_friendships: AtomicUsize::new(0),
            total_messages: AtomicUsize::new(0),
            start_time: Instant::now(),
            anomalies: Mutex::new(Vec::new()),
        }
    }

    /// Record an action execution
    pub fn record_action(&self, action: &RandomAction, result: &ActionResult) {
        let action_type = RandomActionType::from(action);

        // Increment action count
        *self.action_counts.entry(action_type).or_default() += 1;

        match result {
            ActionResult::Success { duration, .. } => {
                *self.success_counts.entry(action_type).or_default() += 1;
                self.record_latency(action_type, *duration);
            }
            ActionResult::ExpectedFailure { duration, .. } => {
                *self.failure_counts.entry(action_type).or_default() += 1;
                self.record_latency(action_type, *duration);
            }
            ActionResult::UnexpectedFailure { error, duration } => {
                *self.failure_counts.entry(action_type).or_default() += 1;
                self.unexpected_error_count.fetch_add(1, Ordering::Relaxed);
                self.record_latency(action_type, *duration);

                let record = ErrorRecord {
                    action_type,
                    error: error.clone(),
                    user_id: self.extract_user_id(action),
                };
                self.errors.lock().unwrap().push(record);
            }
        }
    }

    /// Record latency for an action type
    fn record_latency(&self, action_type: RandomActionType, duration: Duration) {
        let mut histogram = self
            .latency_histograms
            .entry(action_type)
            .or_insert_with(|| Histogram::<u64>::new(3).expect("Failed to create histogram"));

        let millis = duration.as_millis() as u64;
        if let Err(e) = histogram.record(millis) {
            tracing::warn!("Failed to record latency: {}", e);
        }
    }

    /// Extract user ID from action if available
    fn extract_user_id(&self, action: &RandomAction) -> Option<u64> {
        match action {
            RandomAction::SendMessage { user_id, .. }
            | RandomAction::AddFriend { user_id, .. }
            | RandomAction::GetSessionInfo { user_id, .. }
            | RandomAction::GetAccountInfo { user_id, .. } => Some(user_id.0),
            _ => None,
        }
    }

    /// Record a state checkpoint
    pub fn record_checkpoint(&self, stats: StateStats) {
        // Update peaks
        self.peak_users
            .fetch_max(stats.user_count, Ordering::Relaxed);
        self.peak_sessions
            .fetch_max(stats.session_count, Ordering::Relaxed);
        self.peak_friendships
            .fetch_max(stats.friendship_count, Ordering::Relaxed);
        self.total_messages
            .store(stats.message_count, Ordering::Relaxed);
    }

    /// Record an anomaly
    pub fn record_anomaly(&self, severity: AnomalySeverity, message: String) {
        let anomaly = Anomaly { severity, message };
        self.anomalies.lock().unwrap().push(anomaly);
    }

    /// Get total action count
    pub fn total_actions(&self) -> usize {
        self.action_counts.iter().map(|entry| *entry.value()).sum()
    }

    /// Get total success count
    pub fn total_successes(&self) -> usize {
        self.success_counts.iter().map(|entry| *entry.value()).sum()
    }

    /// Get total failure count
    pub fn total_failures(&self) -> usize {
        self.failure_counts.iter().map(|entry| *entry.value()).sum()
    }

    /// Get elapsed time since start
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Print progress report
    pub fn print_progress(&self, stats: &StateStats) {
        let elapsed = self.elapsed();
        let elapsed_secs = elapsed.as_secs();
        let total = self.total_actions();

        println!(
            "\r[{:02}:{:02}:{:02}] Actions: {} | Users: {} | Sessions: {} | Friendships: {} | Messages: {} | Errors: {}",
            elapsed_secs / 3600,
            (elapsed_secs % 3600) / 60,
            elapsed_secs % 60,
            total,
            stats.user_count,
            stats.session_count,
            stats.friendship_count,
            stats.message_count,
            self.unexpected_error_count.load(Ordering::Relaxed)
        );
        std::io::stdout().flush().ok();
    }

    /// Generate final report
    pub fn generate_report(&self) -> RandomTestReport {
        let total = self.total_actions();
        let success = self.total_successes();
        let failures = self.total_failures();
        let success_rate = if total > 0 {
            (success as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        // Collect action distribution
        let mut action_distribution = Vec::new();
        for entry in self.action_counts.iter() {
            let action_type = *entry.key();
            let count = *entry.value();
            let successes = self
                .success_counts
                .get(&action_type)
                .map(|v| *v.value())
                .unwrap_or(0);
            let action_success_rate = if count > 0 {
                (successes as f64 / count as f64) * 100.0
            } else {
                0.0
            };

            // Get average latency
            let avg_latency = self
                .latency_histograms
                .get(&action_type)
                .map(|h| h.value_at_quantile(0.5))
                .unwrap_or(0);

            action_distribution.push(ActionStats {
                action_type,
                count,
                successes,
                failures: count - successes,
                success_rate: action_success_rate,
                avg_latency_ms: avg_latency,
            });
        }

        // Sort by count descending
        action_distribution.sort_by_key(|a| std::cmp::Reverse(a.count));

        // Get recent errors
        let recent_errors = self.errors.lock().unwrap().clone();

        // Get anomalies
        let anomalies = self.anomalies.lock().unwrap().clone();

        RandomTestReport {
            duration: self.elapsed(),
            total_actions: total,
            total_successes: success,
            total_failures: failures,
            success_rate,
            action_distribution,
            unexpected_errors: self.unexpected_error_count.load(Ordering::Relaxed),
            peak_users: self.peak_users.load(Ordering::Relaxed),
            peak_sessions: self.peak_sessions.load(Ordering::Relaxed),
            peak_friendships: self.peak_sessions.load(Ordering::Relaxed),
            total_messages: self.total_messages.load(Ordering::Relaxed),
            recent_errors,
            anomalies,
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Action statistics for report
#[derive(Clone, Debug, serde::Serialize)]
pub struct ActionStats {
    pub action_type: RandomActionType,
    pub count: usize,
    pub successes: usize,
    pub failures: usize,
    pub success_rate: f64,
    pub avg_latency_ms: u64,
}

/// Final test report
#[derive(Clone, Debug, serde::Serialize)]
pub struct RandomTestReport {
    pub duration: Duration,
    pub total_actions: usize,
    pub total_successes: usize,
    pub total_failures: usize,
    pub success_rate: f64,
    pub action_distribution: Vec<ActionStats>,
    pub unexpected_errors: usize,
    pub peak_users: usize,
    pub peak_sessions: usize,
    pub peak_friendships: usize,
    pub total_messages: usize,
    pub recent_errors: Vec<ErrorRecord>,
    pub anomalies: Vec<Anomaly>,
}

impl std::fmt::Display for RandomTestReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = Table::new();
        table.load_preset(comfy_table::presets::UTF8_FULL);

        // Header
        table.set_header(vec![
            "════════════════════════════════════════════════════════════════",
        ]);
        table.add_row(vec![format!(
            "                    Random Test Report                        "
        )]);
        table.add_row(vec![
            "════════════════════════════════════════════════════════════════",
        ]);

        // Duration and totals
        let elapsed = self.duration.as_secs();
        table.add_row(vec![format!(
            "Duration: {:02}:{:02}:{:02} | Total Actions: {} | Success Rate: {:.1}%",
            elapsed / 3600,
            (elapsed % 3600) / 60,
            elapsed % 60,
            self.total_actions,
            self.success_rate
        )]);

        table.add_row(vec![""]);

        // Action distribution (top 10)
        table.set_header(vec!["Action", "Count", "Success %", "Avg Latency"]);
        for stats in self.action_distribution.iter().take(10) {
            table.add_row(vec![
                format!("{}", stats.action_type),
                format!("{}", stats.count),
                format!("{:.1}%", stats.success_rate),
                format!("{}ms", stats.avg_latency_ms),
            ]);
        }

        table.add_row(vec![""]);

        // Peak state
        table.set_header(vec!["Metric", "Peak Value"]);
        table.add_row(vec![format!("Users"), format!("{}", self.peak_users)]);
        table.add_row(vec![format!("Sessions"), format!("{}", self.peak_sessions)]);
        table.add_row(vec![
            format!("Friendships"),
            format!("{}", self.peak_friendships),
        ]);
        table.add_row(vec![
            format!("Messages"),
            format!("{}", self.total_messages),
        ]);

        table.add_row(vec![""]);

        // Errors
        table.set_header(vec!["Error Type", "Count"]);
        table.add_row(vec![
            format!("Unexpected Errors"),
            format!("{}", self.unexpected_errors),
        ]);

        if !self.anomalies.is_empty() {
            table.add_row(vec![""]);
            table.set_header(vec!["Anomalies Detected"]);
            for anomaly in &self.anomalies {
                table.add_row(vec![
                    format!("{:?}", anomaly.severity),
                    anomaly.message.clone(),
                ]);
            }
        }

        write!(f, "{}", table)?;

        // Recent errors (if any)
        if !self.recent_errors.is_empty() {
            writeln!(f, "\n\nRecent Errors (last 10):")?;
            for error in self.recent_errors.iter().take(10) {
                writeln!(
                    f,
                    "  [{}] {}: {}",
                    error.action_type,
                    error.user_id.unwrap_or(0),
                    error.error
                )?;
            }
        }

        Ok(())
    }
}
