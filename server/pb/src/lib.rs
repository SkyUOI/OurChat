#![cfg(not(doctest))]

pub mod service;
pub mod time;

pub mod google {
    pub mod protobuf {
        include!("./generated/google.protobuf.rs");
    }
}
