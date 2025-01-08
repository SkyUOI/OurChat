//! Define constants
//! TODO: use new type for roles and permissions

use crate::impl_newtype_int;
use size::Size;
use std::{io::IsTerminal, path::PathBuf, sync::LazyLock, time::Duration};

/// OCID Length
pub const OCID_LEN: usize = 10;

/// default ip
pub const DEFAULT_IP: &str = "0.0.0.0";

pub fn default_ip() -> String {
    String::from(DEFAULT_IP)
}

/// default port
pub const DEFAULT_PORT: u16 = 7777;
/// http server default port
pub const DEFAULT_HTTP_PORT: u16 = 7778;
pub const APP_NAME: &str = "OurChat";
pub const LOG_ENV_VAR: &str = "OURCHAT_LOG";
pub const LOG_OUTPUT_DIR: &str = "log/";
pub const CONFIG_FILE_ENV_VAR: &str = "OURCHAT_CONFIG_FILE";
// TODO:add this to config file
pub const VERIFY_EMAIL_EXPIRE: Duration = Duration::from_mins(5);

// define ID type to fit many types of databases
impl_newtype_int!(ID, u64, serde::Serialize, serde::Deserialize);
pub type SessionID = ID;
pub type MsgID = ID;

macro impl_from($ty:ty) {
    impl From<$ty> for ID {
        fn from(value: $ty) -> Self {
            ID(value.try_into().unwrap())
        }
    }

    impl From<ID> for $ty {
        fn from(value: ID) -> Self {
            value.0 as $ty
        }
    }
}

impl_from!(i32);
impl_from!(i64);
impl_from!(u32);
impl_from!(u64);

impl From<ID> for sea_orm::Value {
    fn from(value: ID) -> Self {
        Self::BigUnsigned(Some(*value))
    }
}

/// ocid type
pub type OCID = String;

/// default clear interval
pub const fn default_clear_interval() -> u64 {
    1
}

/// default file save days
pub const fn default_file_save_days() -> u64 {
    10
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

pub const fn default_http_port() -> u16 {
    DEFAULT_HTTP_PORT
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

pub const fn default_verification_expire_days() -> u64 {
    3
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

pub const fn default_ssl() -> bool {
    false
}

pub const fn default_enable_email() -> bool {
    false
}

pub fn default_files_storage_path() -> PathBuf {
    PathBuf::from("files_storage/")
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

pub static STDIN_AVAILABLE: LazyLock<bool> = LazyLock::new(|| std::io::stdin().is_terminal());
