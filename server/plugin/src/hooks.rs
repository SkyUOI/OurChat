//! Plugin hook system
//!
//! Defines hook types and registry for plugin event handling

use crate::error::PluginResult;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

/// Hook execution result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HookResult {
    /// Continue with normal execution
    Continue,
    /// Stop execution and return error
    Stop(String),
    /// Modify the data and continue (JSON value)
    Modify(serde_json::Value),
}

/// Message hook context - passed to message-related hooks
#[derive(Debug, Clone)]
pub struct MessageHookContext {
    pub sender_id: Option<u64>,
    pub session_id: Option<u64>,
    pub msg_data: Vec<u8>,
    pub is_encrypted: bool,
}

/// User event context
#[derive(Debug, Clone)]
pub struct UserEventContext {
    pub user_id: u64,
    pub username: String,
    pub email: String,
}

/// Session event context
#[derive(Debug, Clone)]
pub struct SessionEventContext {
    pub session_id: u64,
    pub creator_id: u64,
    pub session_type: i32,
}

/// Hook types that plugins can register for
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookType {
    /// Called before a message is sent
    PreMessageSend,
    /// Called after a message is successfully sent
    PostMessageSend,
    /// Called when a new user is created
    UserCreated,
    /// Called when a user logs in
    UserLogin,
    /// Called when friends are added
    FriendAdded,
    /// Called when a new session is created
    SessionCreated,
}

/// Represents a registered hook from a plugin
#[derive(Debug, Clone)]
pub struct Hook {
    pub plugin_id: String,
    pub hook_type: HookType,
    pub func_name: String,
}

/// Registry for all plugin hooks
#[derive(Debug)]
pub struct HookRegistry {
    hooks: DashMap<HookType, Vec<Hook>>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self {
            hooks: DashMap::new(),
        }
    }

    /// Register a hook
    pub fn register(&self, hook: Hook) -> PluginResult<()> {
        let mut hooks = self.hooks.entry(hook.hook_type).or_default();
        hooks.push(hook);
        Ok(())
    }

    /// Get all hooks for a specific type
    pub fn get_hooks(&self, hook_type: HookType) -> Vec<Hook> {
        self.hooks
            .get(&hook_type)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    /// Remove all hooks for a plugin
    pub fn remove_plugin_hooks(&self, plugin_id: &str) {
        for mut hooks in self.hooks.iter_mut() {
            hooks.retain(|h| h.plugin_id != plugin_id);
        }
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}
