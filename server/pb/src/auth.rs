pub mod v1 {
    tonic::include_proto!("service.auth.v1");
}

pub mod register {
    pub mod v1 {
        tonic::include_proto!("service.auth.register.v1");
    }
}

pub mod email_verify {
    pub mod v1 {
        tonic::include_proto!("service.auth.email_verify.v1");
    }
}

pub mod authorize {
    pub mod v1 {
        tonic::include_proto!("service.auth.authorize.v1");
    }
}
