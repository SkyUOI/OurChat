//! OurChat Plugin SDK
//!
//! This SDK provides helper traits and types for developing OurChat plugins.
//!
//! Plugin authors should use `wit_bindgen::generate!()` in their plugin
//! to generate the actual WIT bindings, then implement the traits provided
//! by this SDK.

// Include the bindings placeholder (not actual generated code)
mod bindings;

/// Convenience logging functions
pub mod logging {
    /// Log a trace message
    pub fn trace(msg: &str) {
        tracing::trace!("{}", msg);
    }

    /// Log a debug message
    pub fn debug(msg: &str) {
        tracing::debug!("{}", msg);
    }

    /// Log an info message
    pub fn info(msg: &str) {
        tracing::info!("{}", msg);
    }

    /// Log a warning message
    pub fn warn(msg: &str) {
        tracing::warn!("{}", msg);
    }

    /// Log an error message
    pub fn error(msg: &str) {
        tracing::error!("{}", msg);
    }
}

/// Configuration functions
pub mod config {
    use std::collections::HashMap;

    static PLUGIN_CONFIG: std::sync::OnceLock<std::sync::RwLock<HashMap<String, String>>> =
        std::sync::OnceLock::new();

    fn get_config() -> &'static std::sync::RwLock<HashMap<String, String>> {
        PLUGIN_CONFIG.get_or_init(|| std::sync::RwLock::new(HashMap::new()))
    }

    /// Get a configuration value
    pub fn get(key: &str) -> Option<String> {
        get_config().read().ok()?.get(key).cloned()
    }

    /// Set a configuration value
    pub fn set(key: String, value: String) -> Result<(), String> {
        get_config()
            .write()
            .map_err(|e| e.to_string())?
            .insert(key, value);
        Ok(())
    }

    /// Get all configuration keys
    pub fn list_keys() -> Vec<String> {
        get_config()
            .read()
            .map(|config| config.keys().cloned().collect())
            .unwrap_or_default()
    }
}

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::logging::*;
    pub use crate::config::*;
}
