use crate::consts::{MessageType, OCID};
use crate::pb::get_info::RequestValues;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::LazyLock};

pub static OWNER_PRIVILEGE: LazyLock<HashSet<RequestValues>> = LazyLock::new(|| {
    collection_literals::collection! {
        RequestValues::Sessions,
        RequestValues::Friends,
        RequestValues::UpdateTime,
        RequestValues::Email,
    }
});
