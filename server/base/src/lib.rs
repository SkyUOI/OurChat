#![feature(decl_macro)]
#![feature(duration_constructors)]

pub mod consts;
pub mod cryption;
pub mod database;
pub mod email_client;
pub mod log;
pub mod rabbitmq;
pub mod setting;
pub mod shutdown;
pub mod types;

shadow_rs::shadow!(build);
