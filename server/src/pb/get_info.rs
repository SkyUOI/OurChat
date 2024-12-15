use std::{collections::HashSet, sync::LazyLock};

tonic::include_proto!("get_info");

pub static OWNER_PRIVILEGE: LazyLock<HashSet<RequestValues>> = LazyLock::new(|| {
    collection_literals::collection! {
        RequestValues::Sessions,
        RequestValues::Friends,
        RequestValues::UpdateTime,
        RequestValues::Email,
    }
});
