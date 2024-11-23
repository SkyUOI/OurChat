//! Define constants

use base::impl_newtype_int;
use serde::{Deserialize, Serialize, Serializer};
use std::{fmt::Display, io::IsTerminal, str::FromStr, sync::LazyLock, time::Duration};

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

pub static STDIN_AVAILABLE: LazyLock<bool> = LazyLock::new(|| std::io::stdin().is_terminal());

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
