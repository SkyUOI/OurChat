//! OurChat Plugin SDK
//!
//! This SDK provides type-safe bindings for developing OurChat plugins.

/// Logging functions
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

/// Hook types and traits
pub mod hooks {
    /// Hook execution result
    #[derive(Debug, Clone, PartialEq)]
    pub enum HookResult {
        /// Continue normal execution
        Continue,
        /// Stop execution with reason
        Stop(String),
        /// Modify data (JSON string)
        Modify(String),
    }

    /// Message hook context
    #[derive(Debug, Clone)]
    pub struct MessageContext {
        pub sender_id: Option<u64>,
        pub session_id: Option<u64>,
        pub msg_data: Vec<u8>,
        pub is_encrypted: bool,
    }

    /// User event context
    #[derive(Debug, Clone)]
    pub struct UserContext {
        pub user_id: u64,
        pub username: String,
        pub email: String,
    }

    /// Session event context
    #[derive(Debug, Clone)]
    pub struct SessionContext {
        pub session_id: u64,
        pub creator_id: u64,
        pub session_type: i32,
    }

    /// Plugin lifecycle trait
    pub trait PluginLifecycle {
        /// Called when plugin is loaded
        fn on_load(&mut self) -> Result<(), String> {
            Ok(())
        }

        /// Called when plugin is enabled
        fn on_enable(&mut self) -> Result<(), String> {
            Ok(())
        }

        /// Called when plugin is disabled
        fn on_disable(&mut self) -> Result<(), String> {
            Ok(())
        }

        /// Called when plugin is unloaded
        fn on_unload(&mut self) {}
    }

    /// Message hooks trait
    pub trait MessageHooks {
        /// Called before a message is sent
        fn on_message_send(&mut self, _ctx: &MessageContext) -> HookResult {
            HookResult::Continue
        }

        /// Called after a message is sent
        fn on_message_sent(&mut self, _msg_id: u64) {}

        /// Called when a new user is created
        fn on_user_created(&mut self, _ctx: &UserContext) {}

        /// Called when a user logs in
        fn on_user_login(&mut self, _ctx: &UserContext) {}

        /// Called when friends are added
        fn on_friend_added(&mut self, _user_id: u64, _friend_id: u64, _session_id: u64) {}

        /// Called when a new session is created
        fn on_session_created(&mut self, _ctx: &SessionContext) {}
    }
}

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::logging::*;
    pub use crate::config::*;
    pub use crate::hooks::*;
}
