use base::consts::OCID;
use utils::impl_newtype_string;

impl_newtype_string!(MatrixUserId, serde::Serialize, serde::Deserialize);

impl MatrixUserId {
    pub fn from_ocid(ocid: &OCID, domain: impl AsRef<str>) -> Self {
        Self(format!("@{}:{}", ocid, domain.as_ref()))
    }
}
