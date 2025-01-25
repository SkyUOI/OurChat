pub mod v1 {
    include!("../../generated/service.ourchat.msg_delivery.v1.rs");

    pub type BundleMsgs = Vec<OneMsg>;
}

pub mod recall {
    pub mod v1 {
        include!("../../generated/service.ourchat.msg_delivery.recall.v1.rs");
    }
}
