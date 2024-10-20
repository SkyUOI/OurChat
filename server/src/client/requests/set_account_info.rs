use crate::{client::basic::SetAccountValues, consts::MessageType};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

pub static CHANGE_PUBLIC_TIME: LazyLock<HashSet<SetAccountValues>> =
    LazyLock::new(|| HashSet::from_iter([SetAccountValues::UserName]));

#[derive(Debug, Serialize, Deserialize)]
pub struct SetAccountRequest {
    pub code: MessageType,
    pub data: HashMap<SetAccountValues, serde_json::Value>,
}

impl SetAccountRequest {
    pub fn new(data: HashMap<SetAccountValues, serde_json::Value>) -> Self {
        Self {
            code: MessageType::SetAccountInfo,
            data,
        }
    }
}
