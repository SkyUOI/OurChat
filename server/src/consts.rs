//! 常量

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
/// id类型
pub type ID = u64;
/// ocid type
pub type OCID = String;

/// 默认清理间隔天数
pub const fn default_clear_interval() -> u64 {
    1
}

/// 文件默认保存天数
pub const fn default_file_save_days() -> u64 {
    10
}

pub const fn default_enable_cmd() -> bool {
    true
}
