pub mod server;

pub mod v1 {
    include!("../generated/service.basic.v1.rs");
}

pub mod support {
    pub mod v1 {
        include!("../generated/service.basic.support.v1.rs");
    }
}

pub mod preset_user_status {
    pub mod v1 {
        include!("../generated/service.basic.preset_user_status.v1.rs");
    }
}
