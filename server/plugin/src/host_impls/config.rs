//! Configuration host implementation
//!
//! Provides plugin configuration access via the WIT config interface.

use crate::engine::PluginState;
use parking_lot::RwLock;
use std::sync::Arc;

/// Host implementation for the config interface
pub struct ConfigHost {
    state: Arc<RwLock<PluginState>>,
}

impl ConfigHost {
    pub fn new(state: Arc<RwLock<PluginState>>) -> Self {
        Self { state }
    }

    fn get_config(&self, key: &str) -> Option<serde_json::Value> {
        self.state.read().config.read().get(key)?.as_object()?.get(key).cloned()
    }
}

/// Implement the config interface from WIT
impl ConfigHost {
    pub fn get_string(&self, key: String) -> Option<String> {
        let state = self.state.read();
        let config = state.config.read();
        config.get(&key)?.as_str().map(String::from)
    }

    pub fn get_int(&self, key: String) -> Option<i64> {
        let state = self.state.read();
        let config = state.config.read();
        config.get(&key)?.as_i64()
    }

    pub fn get_bool(&self, key: String) -> Option<bool> {
        let state = self.state.read();
        let config = state.config.read();
        config.get(&key)?.as_bool()
    }

    pub fn set(&mut self, key: String, value: String) -> Result<(), String> {
        let state = self.state.read();
        let mut config = state.config.write();
        if let Some(obj) = config.as_object_mut() {
            obj.insert(key, serde_json::Value::String(value));
        }
        Ok(())
    }

    pub fn list_keys(&self) -> Vec<String> {
        let state = self.state.read();
        let config = state.config.read();
        if let Some(obj) = config.as_object() {
            obj.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }
}
