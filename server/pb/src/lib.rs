#![cfg(not(doctest))]

pub mod google;
pub mod service;
pub mod time;

pub const FILE_DESCRIPTOR: &[u8] = include_bytes!("./generated/GRPC_FILE_DESCRIPTOR");
