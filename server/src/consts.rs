use num_enum::TryFromPrimitive;
use serde_repr::{Deserialize_repr, Serialize_repr};

pub const OCID_LEN: u32 = 10;

#[derive(Debug, Serialize_repr, Deserialize_repr, TryFromPrimitive, PartialEq, Eq)]
#[repr(i32)]
pub enum RequestType {
    Login = 6,
    LoginRes = 7,
    Register = 4,
    RegisterRes = 5,
    Unregister = 16,
    UnregisterRes = 17,
    ErrorMsg = 18,
}
