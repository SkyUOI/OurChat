#![feature(decl_macro)]
#![feature(duration_constructors)]
#![feature(duration_constructors_lite)]
#![feature(path_file_prefix)]

pub mod consts;
pub mod cryption;
pub mod database;
pub mod email_client;
pub mod log;
pub mod rabbitmq;
pub mod setting;
pub mod shutdown;
pub mod types;
pub mod wrapper;

shadow_rs::shadow!(build);
