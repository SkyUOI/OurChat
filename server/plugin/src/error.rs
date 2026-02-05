//! Plugin error types

use thiserror::Error;

pub type PluginResult<T> = Result<T, PluginError>;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("WASM runtime error: {0}")]
    Wasm(String),

    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Plugin already loaded: {0}")]
    AlreadyLoaded(String),

    #[error("Plugin load failed: {0}")]
    LoadFailed(String),

    #[error("Hook execution error: {0}")]
    HookError(String),

    #[error("Invalid plugin metadata: {0}")]
    InvalidMetadata(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Db(String),

    #[error("Plugin timeout: execution exceeded {0}ms")]
    Timeout(u64),

    #[error("Plugin resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    #[error("Plugin panicked: {0}")]
    Panic(String),
}
