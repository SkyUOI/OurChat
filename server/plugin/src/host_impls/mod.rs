//! Host implementations for WIT interfaces
//!
//! This module contains the host-side implementations of all WIT interfaces
//! that are exposed to plugins.

pub mod logging;
pub mod config;
pub mod database;
pub mod http;
pub mod redis;
pub mod messaging;

pub use logging::LoggingHost;
pub use config::ConfigHost;
pub use database::DatabaseHost;
pub use http::HttpHost;
pub use redis::RedisHost;
pub use messaging::MessagingHost;
