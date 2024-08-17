use crate::consts::{MessageType, OCID};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NewSession {
    pub code: MessageType,
    pub members: Vec<OCID>,
}

impl NewSession {
    pub fn new(members: Vec<OCID>) -> Self {
        Self {
            code: MessageType::NewSession,
            members,
        }
    }
}
