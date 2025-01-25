//! Database

pub mod file_storage;
pub mod helper;
pub mod messages;
pub mod redis;
pub mod session;
pub mod user;

/// Initialize the database layer
pub fn init_db_system() {
    tracing::info!("Init db system");
}
