//! Define constants

use pb::service::basic::server::v1::ServerVersion;
use size::Size;
use std::{path::PathBuf, str::FromStr, sync::LazyLock, time::Duration};
use utils::{impl_newtype_int, impl_newtype_string, impl_redis_value_from_for_newint};

/// OCID Length
pub const OCID_LEN: usize = 10;

/// Defined in snowflake algorithm
pub const MACHINE_ID_MAX: u64 = 1024 - 1;

/// default ip
pub const DEFAULT_IP: &str = "0.0.0.0";

pub fn default_ip() -> String {
    String::from(DEFAULT_IP)
}

/// default port
pub const DEFAULT_PORT: u16 = 7777;
pub const APP_NAME: &str = "OurChat";
pub const LOG_ENV_VAR: &str = "OURCHAT_LOG";
pub const LOG_OUTPUT_DIR: &str = "log/";
pub const CONFIG_FILE_ENV_VAR: &str = "OURCHAT_CONFIG_FILE";

// Log file name
// Main Server
pub static OURCHAT_LOG_PREFIX: &str = "ourchat";
// Http Server

pub const fn default_verify_email_expiry() -> Duration {
    Duration::from_mins(5)
}

pub const SERVER_INFO_JSON_VERSION: u64 = 1;

pub const fn default_add_friend_request_expiry() -> Duration {
    Duration::from_days(3)
}

// define ID type to fit many types of databases
impl_newtype_int!(ID, u64,);
impl_redis_value_from_for_newint!(ID);
impl_newtype_int!(SessionID, u64,);
impl_newtype_int!(MsgID, u64,);

pub macro impl_from($from:path, $ty:ty) {
    impl From<$ty> for $from {
        fn from(value: $ty) -> Self {
            $from(value.try_into().expect("numeric conversion failed"))
        }
    }

    impl From<$from> for $ty {
        fn from(value: $from) -> Self {
            value.0 as $ty
        }
    }
}

pub macro impl_from_all_ints($from:path) {
    impl_from!($from, i32);
    impl_from!($from, i64);
    impl_from!($from, u32);
    impl_from!($from, u64);
}

impl_from_all_ints!(ID);
impl_from_all_ints!(SessionID);

impl From<ID> for sea_orm::Value {
    fn from(value: ID) -> Self {
        Self::BigUnsigned(Some(*value))
    }
}

impl From<SessionID> for sea_orm::Value {
    fn from(value: SessionID) -> Self {
        Self::BigUnsigned(Some(*value))
    }
}

// ocid type
impl_newtype_string!(OCID, serde::Serialize, serde::Deserialize);

impl From<OCID> for sea_orm::Value {
    fn from(value: OCID) -> Self {
        Self::String(Some(Box::new(value.0)))
    }
}

impl From<&OCID> for sea_orm::Value {
    fn from(value: &OCID) -> Self {
        Self::String(Some(Box::new(value.0.clone())))
    }
}

/// default clear interval
pub fn default_clear_interval() -> croner::Cron {
    croner::Cron::from_str("0 0 0 * *").expect("incorrect initial cron")
}

/// default file save days
pub const fn default_file_save_time() -> Duration {
    Duration::from_days(10)
}

/// default log clean duration
pub const fn default_log_clean_duration() -> Duration {
    Duration::from_days(30)
}

pub const fn default_log_keep() -> Duration {
    Duration::from_days(3)
}

/// whether to enable cmd
pub const fn default_enable_cmd() -> bool {
    true
}

pub const fn default_friends_number_limit() -> u32 {
    5000
}

pub const fn default_enable_cmd_stdin() -> bool {
    true
}

pub const fn default_port() -> u16 {
    DEFAULT_PORT
}

pub const fn default_debug_console_port() -> u16 {
    7776
}

pub const fn default_debug_console() -> bool {
    true
}

pub const fn default_fetch_msg_page_size() -> u64 {
    2000
}

pub const fn default_verification_expire_time() -> Duration {
    Duration::from_days(3)
}

pub const fn default_user_defined_status_expire_time() -> Duration {
    Duration::from_hours(24)
}

pub const fn default_t_cost() -> u32 {
    2
}

pub const fn default_m_cost() -> u32 {
    19456
}

pub const fn default_p_cost() -> u32 {
    1
}

pub const fn default_output_len() -> Option<usize> {
    None
}

pub const fn default_tls() -> bool {
    false
}

pub const fn default_client_certificate_required() -> bool {
    false
}

pub const fn default_enable_email() -> bool {
    false
}

pub fn default_files_storage_path() -> PathBuf {
    PathBuf::from("files_storage/")
}

pub const fn default_enable_file_cache() -> bool {
    true
}

pub const fn default_enable_hierarchical_storage() -> bool {
    true
}

pub const fn default_enable_file_deduplication() -> bool {
    true
}

pub fn default_cache_max_size() -> Size {
    Size::from_mebibytes(100)
}

/// default user files store limit(MB)
pub fn default_user_files_store_limit() -> Size {
    Size::from_mebibytes(100)
}

pub const fn default_leader_node() -> bool {
    true
}

pub const fn default_single_instance() -> bool {
    true
}

pub const fn default_http_run_migration() -> bool {
    false
}

pub const fn default_enable_matrix() -> bool {
    false
}

pub const fn default_password_strength_limit() -> zxcvbn::Score {
    zxcvbn::Score::One
}

pub const fn default_network_cmd_port() -> u16 {
    7779
}

pub const fn default_room_key_duration() -> Duration {
    Duration::from_days(30)
}

pub const fn default_keep_voip_room_keep_duration() -> Duration {
    Duration::from_mins(10)
}

pub const fn default_rate_limit_enable() -> bool {
    true
}

pub const fn default_rate_limit_burst() -> u32 {
    16
}

pub const fn default_rate_limit_replenish_duration() -> Duration {
    Duration::from_millis(500)
}

pub const fn default_web_panel_enable() -> bool {
    true
}

/// Placeholder
pub fn default_web_panel_dist_path() -> PathBuf {
    PathBuf::default()
}

pub const fn default_lock_account_after_failed_logins() -> u32 {
    5
}

pub const fn default_lock_account_duration() -> Duration {
    Duration::from_mins(15)
}

pub mod option {
    pub const fn default_network_cmd_port() -> Option<u16> {
        Some(super::default_network_cmd_port())
    }
}

pub static SERVER_INFO_PATH: &str = "server_info.json";

pub static VERSION_SPLIT: LazyLock<ServerVersion> = LazyLock::new(|| {
    let ver = crate::build::PKG_VERSION.split('.').collect::<Vec<_>>();
    ServerVersion {
        major: ver
            .first()
            .and_then(|v| v.parse().ok())
            .expect("invalid version format: missing or invalid major version"),
        minor: ver
            .get(1)
            .and_then(|v| v.parse().ok())
            .expect("invalid version format: missing or invalid minor version"),
        patch: ver
            .get(2)
            .and_then(|v| v.parse().ok())
            .expect("invalid version format: missing or invalid patch version"),
    }
});

pub const JWT_HEADER: &str = "authorization";

// OAuth defaults
pub const fn default_oauth_enable() -> bool {
    false
}

pub fn default_oauth_github_client_id() -> String {
    "".to_string()
}

pub fn default_oauth_github_client_secret() -> String {
    "".to_string()
}

pub const fn default_require_email_verification() -> bool {
    false
}

/// Default STUN servers for NAT traversal
/// Uses Google's public STUN servers for development
pub fn default_stun_servers() -> Vec<String> {
    vec![
        "stun:stun.l.google.com:19302".to_string(),
        "stun:stun1.l.google.com:19302".to_string(),
    ]
}

pub fn default_turn_server_url() -> String {
    "".to_string()
}

pub fn default_turn_username() -> String {
    "".to_string()
}

pub fn default_turn_password() -> String {
    "".to_string()
}

pub fn default_turn_ttl() -> u64 {
    86400 // 24 hours in seconds
}
