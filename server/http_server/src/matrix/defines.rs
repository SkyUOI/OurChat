use base::consts::OCID;
use base::impl_newtype_string;
use serde::{Deserialize, Serialize};

impl_newtype_string!(MatrixUserId, serde::Serialize, serde::Deserialize);

impl MatrixUserId {
    pub fn from_ocid(ocid: &OCID, domain: impl AsRef<str>) -> Self {
        Self(format!("@{}:{}", ocid, domain.as_ref()))
    }
}
