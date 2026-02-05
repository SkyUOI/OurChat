//! Plugin registry and metadata management
//!
//! Tracks loaded plugins, their state, and configuration

use crate::error::{PluginError, PluginResult};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Plugin state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginState {
    Loaded,
    Enabled,
    Disabled,
    Failed,
}

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub file_path: PathBuf,
    pub state: PluginState,
    pub enabled_at: Option<DateTime<Utc>>,
    pub loaded_at: DateTime<Utc>,
    pub config: serde_json::Value,
    pub error: Option<String>,
}

/// Registry for all plugins
#[derive(Debug)]
pub struct PluginRegistry {
    plugins: DashMap<String, PluginMetadata>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: DashMap::new(),
        }
    }

    /// Register a plugin
    pub fn register(&self, metadata: PluginMetadata) -> PluginResult<()> {
        if self.plugins.contains_key(&metadata.id) {
            return Err(PluginError::AlreadyLoaded(metadata.id));
        }
        self.plugins.insert(metadata.id.clone(), metadata);
        Ok(())
    }

    /// Get plugin metadata by ID
    pub fn get(&self, id: &str) -> Option<PluginMetadata> {
        self.plugins.get(id).map(|p| p.clone())
    }

    /// Get all plugins
    pub fn list(&self) -> Vec<PluginMetadata> {
        self.plugins.iter().map(|p| p.clone()).collect()
    }

    /// Get plugins by state
    pub fn list_by_state(&self, state: PluginState) -> Vec<PluginMetadata> {
        self.plugins
            .iter()
            .filter(|p| p.state == state)
            .map(|p| p.clone())
            .collect()
    }

    /// Update plugin state
    pub fn update_state(&self, id: &str, state: PluginState) -> PluginResult<()> {
        let mut metadata = self
            .plugins
            .get_mut(id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;

        metadata.state = state;
        if state == PluginState::Enabled {
            metadata.enabled_at = Some(Utc::now());
        }

        Ok(())
    }

    /// Update plugin config
    pub fn update_config(&self, id: &str, config: serde_json::Value) -> PluginResult<()> {
        let mut metadata = self
            .plugins
            .get_mut(id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;

        metadata.config = config;
        Ok(())
    }

    /// Set error on a plugin
    pub fn set_error(&self, id: &str, error: String) -> PluginResult<()> {
        let mut metadata = self
            .plugins
            .get_mut(id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;

        metadata.error = Some(error);
        metadata.state = PluginState::Failed;

        Ok(())
    }

    /// Remove a plugin from registry
    pub fn remove(&self, id: &str) -> PluginResult<()> {
        self.plugins
            .remove(id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;
        Ok(())
    }

    /// Check if a plugin exists
    pub fn contains(&self, id: &str) -> bool {
        self.plugins.contains_key(id)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
