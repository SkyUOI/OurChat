//! 常量

use std::num::TryFromIntError;

use base::{impl_newtype, impl_newtype_int};
use num_enum::TryFromPrimitive;
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::db::DbType;

/// OCID 长度
pub const OCID_LEN: usize = 10;

/// 往返的消息类型
#[derive(Debug, Serialize_repr, Deserialize_repr, TryFromPrimitive, PartialEq, Eq)]
#[repr(i32)]
pub enum MessageType {
    Login = 6,
    LoginRes = 7,
    Register = 4,
    RegisterRes = 5,
    Unregister = 16,
    UnregisterRes = 17,
    ErrorMsg = 18,
    NewSession = 8,
    NewSessionResponse = 9,
    GetStatus = 12,
    Upload = 21,
    UploadRes = 22,
}

/// 默认ip
pub const DEFAULT_IP: &str = "127.0.0.1";
/// 默认端口
pub const DEFAULT_PORT: u16 = 7777;
/// http服务器默认端口
pub const DEFAULT_HTTP_PORT: u16 = 7778;
/// 默认采用的数据库类型
pub const DB_TYPE: DbType = DbType::Mysql;

/// 时间戳类型(与chrono不一致)
pub type TimeStamp = u64;
// define ID type to fit many types of databases
impl_newtype!(ID, u64, #[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default, serde::Serialize, serde::Deserialize)]);

impl From<u64> for ID {
    fn from(value: u64) -> Self {
        ID(value)
    }
}

impl From<i64> for ID {
    fn from(value: i64) -> Self {
        ID(value.try_into().unwrap())
    }
}

impl From<ID> for i64 {
    fn from(value: ID) -> Self {
        value.0 as i64
    }
}

impl From<ID> for u64 {
    fn from(value: ID) -> Self {
        value.0
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

/// default user files store limit
pub const fn default_user_files_store_limit() -> u64 {
    100
}
