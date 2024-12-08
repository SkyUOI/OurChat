pub mod v1 {
    use std::{collections::HashSet, sync::LazyLock};

    tonic::include_proto!("service.ourchat.get_account_info.v1");

    pub static OWNER_PRIVILEGE: LazyLock<HashSet<RequestValues>> = LazyLock::new(|| {
        collection_literals::collection! {
            RequestValues::Sessions,
            RequestValues::Friends,
            RequestValues::UpdateTime,
            RequestValues::Email,
        }
    });
}