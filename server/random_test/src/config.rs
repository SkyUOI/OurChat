use serde::{Deserialize, Serialize};
use std::time::Duration;

fn default_ip() -> String {
    "127.0.0.1".to_string()
}

fn default_continue_on_errors() -> bool {
    true
}

fn default_num_users() -> usize {
    50
}

fn default_duration() -> Duration {
    Duration::from_hours(1)
}

fn default_actions_per_second() -> f64 {
    10.0
}

fn default_concurrency() -> usize {
    10
}

fn default_validation_interval() -> Duration {
    Duration::from_secs(30)
}

fn default_metrics_interval() -> Duration {
    Duration::from_secs(30)
}

fn default_max_errors() -> usize {
    100
}

/// Random test configuration loaded from JSON file
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct RandomTestConfig {
    /// Server IP address
    #[serde(default = "default_ip")]
    pub ip: String,

    /// Server port
    #[serde(default = "base::constants::default_port")]
    pub port: u16,

    /// Number of users to create
    #[serde(default = "default_num_users")]
    pub num_users: usize,

    /// Test duration in hours
    #[serde(default = "default_duration", with = "humantime_serde")]
    pub running_duration: Duration,

    /// Action execution rate (actions per second)
    #[serde(default = "default_actions_per_second")]
    pub actions_per_second: f64,

    /// Number of concurrent action executors
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,

    /// Enable detailed logging
    #[serde(default)]
    pub verbose: bool,

    /// State validation interval in seconds
    #[serde(default = "default_validation_interval", with = "humantime_serde")]
    pub validation_interval: Duration,

    /// Metrics reporting interval in seconds
    #[serde(default = "default_metrics_interval", with = "humantime_serde")]
    pub metrics_interval: Duration,

    /// Action weights (embedded)
    #[serde(default)]
    pub action_weights: ActionWeights,

    /// Continue on error (don't stop test)
    #[serde(default = "default_continue_on_errors")]
    pub continue_on_error: bool,

    /// Maximum number of errors before aborting
    #[serde(default = "default_max_errors")]
    pub max_errors: usize,

    /// Random seed for reproducibility (0 for random)
    #[serde(default)]
    pub seed: u64,
}

impl RandomTestConfig {
    pub fn action_interval(&self) -> Duration {
        let millis = (1000.0 / self.actions_per_second) as u64;
        Duration::from_millis(millis)
    }
}

/// Action weights for random generation
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ActionWeights {
    // Messaging actions (most common)
    pub send_message: f64,
    pub fetch_messages: f64,
    pub recall_message: f64,

    // Session operations (less common)
    pub create_session: f64,
    pub delete_session: f64,
    pub join_session: f64,
    pub leave_session: f64,
    pub invite_to_session: f64,
    pub accept_session_invitation: f64,

    // Moderation (rare)
    pub kick_user: f64,
    pub ban_user: f64,
    pub unban_user: f64,
    pub mute_user: f64,
    pub unmute_user: f64,

    // Friend operations
    pub add_friend: f64,
    pub accept_friend_invitation: f64,
    pub delete_friend: f64,

    // File operations (rare)
    pub upload_file: f64,
    pub download_file: f64,
    pub delete_file: f64,

    // Info queries
    pub get_session_info: f64,
    pub get_account_info: f64,

    // Settings changes
    pub set_session_info: f64,
    pub set_account_info: f64,
}

impl Default for ActionWeights {
    fn default() -> Self {
        Self {
            // Messaging - most frequent
            send_message: 100.0,
            fetch_messages: 80.0,
            recall_message: 5.0,

            // Session operations
            create_session: 5.0,
            delete_session: 0.2,
            join_session: 3.0,
            leave_session: 2.0,
            invite_to_session: 5.0,
            accept_session_invitation: 4.0,

            // Moderation - rare
            kick_user: 1.0,
            ban_user: 0.5,
            unban_user: 0.5,
            mute_user: 1.0,
            unmute_user: 1.0,

            // Friend operations
            add_friend: 5.0,
            accept_friend_invitation: 4.0,
            delete_friend: 2.0,

            // File operations - rare
            upload_file: 1.0,
            download_file: 2.0,
            delete_file: 0.5,

            // Info queries
            get_session_info: 30.0,
            get_account_info: 20.0,

            // Settings
            set_session_info: 3.0,
            set_account_info: 1.0,
        }
    }
}
