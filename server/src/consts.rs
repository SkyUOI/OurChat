use serde_repr::{Deserialize_repr, Serialize_repr};

pub const OCID_LEN: u32 = 10;

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum RequestType {
    Login = 6,
    LoginRes = 7,
    Register = 4,
    RegisterRes = 5,
    AccountDeletion = 16,
}
