//! 常量

use num_enum::TryFromPrimitive;
use serde_repr::{Deserialize_repr, Serialize_repr};

/// OCID 长度
pub const OCID_LEN: u32 = 10;

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
}

/// 默认ip
pub const DEFAULT_IP: &str = "127.0.0.1";
/// 默认端口
pub const DEFAULT_PORT: usize = 7777;

/// 时间戳类型(与chrono不一致)
pub type TimeStamp = u64;
