pub mod server;

pub mod v1 {
    include!("../generated/service.basic.v1.rs");
}

pub mod support {
    pub mod v1 {
        include!("../generated/service.basic.support.v1.rs");
    }
}
