pub mod v1 {
    include!("../generated/service.auth.v1.rs");
}

pub mod register {
    pub mod v1 {
        include!("../generated/service.auth.register.v1.rs");
    }
}

pub mod email_verify {
    pub mod v1 {
        include!("../generated/service.auth.email_verify.v1.rs");
    }
}

pub mod authorize {
    pub mod v1 {
        include!("../generated/service.auth.authorize.v1.rs");
    }
}
