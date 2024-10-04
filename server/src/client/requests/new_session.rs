use crate::consts::{MessageType, OCID};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NewSession {
    pub code: MessageType,
    pub members: Vec<OCID>,
    #[serde(default)]
    pub name: String,
    // avatar
}

impl NewSession {
    pub fn new_easiest(members: Vec<OCID>) -> Self {
        Self {
            code: MessageType::NewSession,
            members,
            name: String::default(),
        }
    }

    pub fn new(members: Vec<OCID>, name: String) -> Self {
        Self {
            code: MessageType::NewSession,
            members,
            name,
        }
    }
}
