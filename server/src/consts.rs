//! 常量

use base::impl_newtype_int;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize, Serializer};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::{fmt::Display, io::IsTerminal, str::FromStr, sync::LazyLock};

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
pub const DEFAULT_IP: &str = "0.0.0.0";

pub fn default_ip() -> String {
    String::from(DEFAULT_IP)
}

/// 默认端口
pub const DEFAULT_PORT: u16 = 7777;
/// http服务器默认端口
pub const DEFAULT_HTTP_PORT: u16 = 7778;
/// 默认采用的数据库类型
pub const DB_TYPE: DbType = DbType::MySql;

/// 时间戳类型(与chrono不一致)
pub type TimeStamp = u64;
// define ID type to fit many types of databases
impl_newtype_int!(ID, u64, serde::Serialize, serde::Deserialize);

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

pub const fn default_friends_number_limit() -> u32 {
    5000
}

pub const fn default_enable_cmd_stdin() -> bool {
    true
}

macro impl_to_bytes {
    ($($name:ident, ($($opers:tt)*)),*) => {
        $(
            impl From<$name> for Bt {
                fn from(value: $name) -> Bt {
                    Bt(value.0 * ($($opers)*))
                }
            }

            impl From<Bt> for $name {
                fn from(value: Bt) -> $name {
                    $name(value.0 / ($($opers)*))
                }
            }
        )*
    }
}

impl_newtype_int!(Bt, u64,);
impl_newtype_int!(KBt, u64,);
impl_newtype_int!(MBt, u64,);
impl_newtype_int!(GBt, u64,);
impl_newtype_int!(TBt, u64,);

impl_to_bytes!(
    KBt,
    (1024),
    MBt,
    (1024 * 1024),
    GBt,
    (1024 * 1024 * 1024),
    TBt,
    (1024 * 1024 * 1024 * 1024)
);

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum FileSize {
    B(Bt),
    KB(KBt),
    MB(MBt),
    GB(GBt),
    TB(TBt),
}

impl Display for FileSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileSize::B(v) => write!(f, "{}B", v.0),
            FileSize::KB(v) => write!(f, "{}KB", v.0),
            FileSize::MB(v) => write!(f, "{}MB", v.0),
            FileSize::GB(v) => write!(f, "{}GB", v.0),
            FileSize::TB(v) => write!(f, "{}TB", v.0),
        }
    }
}

impl FromStr for FileSize {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim_end_matches('B');
        if s.is_empty() {
            return Err("empty string".to_string());
        }
        let size_type = s.chars().last().unwrap();
        if size_type.is_alphabetic() {
            match size_type {
                'K' => Ok(FileSize::KB(KBt(match s[..s.len() - 1].parse() {
                    Ok(v) => v,
                    Err(e) => return Err(e.to_string()),
                }))),
                'M' => Ok(FileSize::MB(MBt(match s[..s.len() - 1].parse() {
                    Ok(v) => v,
                    Err(e) => return Err(e.to_string()),
                }))),
                'G' => Ok(FileSize::GB(GBt(match s[..s.len() - 1].parse() {
                    Ok(v) => v,
                    Err(e) => return Err(e.to_string()),
                }))),
                'T' => Ok(FileSize::TB(TBt(match s[..s.len() - 1].parse() {
                    Ok(v) => v,
                    Err(e) => return Err(e.to_string()),
                }))),
                _ => Err(format!("invalid size type \"{}\"", size_type)),
            }
        } else {
            Ok(FileSize::B(Bt(match s.parse() {
                Ok(v) => v,
                Err(e) => return Err(e.to_string()),
            })))
        }
    }
}

impl From<FileSize> for Bt {
    fn from(val: FileSize) -> Self {
        match val {
            FileSize::B(v) => v,
            FileSize::KB(v) => v.into(),
            FileSize::MB(v) => v.into(),
            FileSize::GB(v) => v.into(),
            FileSize::TB(v) => v.into(),
        }
    }
}

impl Serialize for FileSize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

impl<'de> Deserialize<'de> for FileSize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

pub fn change_file_size<T: From<Bt>>(a: impl Into<Bt>) -> T {
    T::from(a.into())
}

/// default user files store limit(MB)
pub const fn default_user_files_store_limit() -> FileSize {
    FileSize::MB(MBt(100))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_size_test() {
        assert_eq!(change_file_size::<KBt>(Bt(1024)), KBt(1));
        assert_eq!(change_file_size::<MBt>(KBt(1024)), MBt(1));
        assert_eq!(change_file_size::<GBt>(MBt(1024)), GBt(1));
        assert_eq!(change_file_size::<TBt>(GBt(1024)), TBt(1));

        assert_eq!(change_file_size::<Bt>(KBt(1024)), Bt(1024 * 1024));
        assert_eq!(change_file_size::<KBt>(MBt(1024)), KBt(1024 * 1024));
        assert_eq!(change_file_size::<MBt>(GBt(1024)), MBt(1024 * 1024));
        assert_eq!(change_file_size::<GBt>(TBt(1024)), GBt(1024 * 1024));

        assert_eq!(change_file_size::<Bt>(MBt(1024)), Bt(1024 * 1024 * 1024));
        assert_eq!(change_file_size::<KBt>(GBt(1024)), KBt(1024 * 1024 * 1024));
        assert_eq!(change_file_size::<MBt>(TBt(1024)), MBt(1024 * 1024 * 1024));
    }

    #[test]
    fn file_size_serde() {
        let test_data = FileSize::MB(MBt(100));
        let s = serde_json::to_string(&test_data).unwrap();
        assert_eq!(s, "\"100MB\"");
        let v: FileSize = serde_json::from_str("\"100MB\"").unwrap();
        assert_eq!(v, test_data);

        let test_data = FileSize::TB(TBt(100));
        let s = serde_json::to_string(&test_data).unwrap();
        assert_eq!(s, "\"100TB\"");
        let v: FileSize = serde_json::from_str("\"100TB\"").unwrap();
        assert_eq!(v, test_data);
    }
}

pub static STDIN_AVAILABLE: LazyLock<bool> = LazyLock::new(|| std::io::stdin().is_terminal());
