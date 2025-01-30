#![cfg(not(doctest))]

pub mod service;

pub mod google {
    pub mod protobuf {
        include!("./generated/google.protobuf.rs");
    }
}
