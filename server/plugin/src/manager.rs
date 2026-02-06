//! Plugin manager
//!
//! Manages plugin lifecycle: loading, enabling, disabling, and executing hooks

use crate::engine::{HookAction, PluginContext, PluginState as EnginePluginState, WasmEngine, LoadedPlugin};
use crate::error::{PluginError, PluginResult};
use crate::hooks::{HookResult, MessageHookContext};
use crate::registry::{PluginMetadata, PluginRegistry, PluginState};
use base::database::DbPool;
use chrono::Utc;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{error, info};

/// Plugin manager - main entry point for the plugin system
pub struct PluginManager {
    engine: Arc<WasmEngine>,
    registry: Arc<PluginRegistry>,
    loaded_plugins: DashMap<String, LoadedPlugin>,
    plugin_directory: PathBuf,
    db_pool: Option<DbPool>,
    redis_pool: Option<deadpool_redis::Pool>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub async fn new(
        plugin_directory: PathBuf,
        db_pool: Option<DbPool>,
        redis_pool: Option<deadpool_redis::Pool>,
    ) -> PluginResult<Self> {
        // Create plugin directory if it doesn't exist
        if !plugin_directory.exists() {
            std::fs::create_dir_all(&plugin_directory)?;
        }

        let engine = Arc::new(WasmEngine::new()?);
        let registry = Arc::new(PluginRegistry::new());

        Ok(Self {
            engine,
            registry,
            loaded_plugins: DashMap::new(),
            plugin_directory,
            db_pool,
            redis_pool,
        })
    }

    /// Load all plugins from the plugin directory
    pub async fn load_all(&self) -> PluginResult<usize> {
        let entries = std::fs::read_dir(&self.plugin_directory)
            .map_err(PluginError::Io)?;

        let mut count = 0;
        for entry_result in entries {
            let entry = entry_result.map_err(PluginError::from)?;
            let path = entry.path();

            // Only load .wasm files
            if path.extension().and_then(|s: &std::ffi::OsStr| s.to_str()) == Some("wasm") {
                match self.load_plugin(&path).await {
                    Ok(_) => {
                        count += 1;
                    }
                    Err(e) => {
                        error!("Failed to load plugin {:?}: {}", path, e);
                    }
                }
            }
        }

        info!("Loaded {} plugins from {}", count, self.plugin_directory.display());
        Ok(count)
    }

    /// Load a single plugin from a file
    pub async fn load_plugin(&self, path: &Path) -> PluginResult<()> {
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| PluginError::LoadFailed("Invalid filename".to_string()))?;

        // Generate plugin ID from filename
        let plugin_id = filename.replace(".wasm", "");

        info!("Loading plugin: {}", plugin_id);

        // Try loading as component first, fall back to legacy module
        let component = match self.engine.load_component(path.to_path_buf()) {
            Ok(component) => {
                info!("Loading plugin {} as component", plugin_id);
                component
            }
            Err(e) => {
                info!("Failed to load as component, trying legacy module: {}", e);
                self.engine.load_module_as_component(path.to_path_buf())?
            }
        };

        // Create plugin state
        let state = EnginePluginState {
            plugin_id: plugin_id.clone(),
            config: Arc::new(RwLock::new(serde_json::Value::Object(serde_json::Map::new()))),
            db_pool: self.db_pool.clone(),
            redis_pool: self.redis_pool.clone(),
            host_data: Arc::new(DashMap::new()),
        };

        // Create plugin context
        // WASI integration will be added once WIT bindings are generated
        let context = PluginContext { state };

        // Instantiate the plugin
        let loaded = self
            .engine
            .instantiate_plugin(component, plugin_id.clone(), context)?;

        // Register metadata
        let metadata = PluginMetadata {
            id: plugin_id.clone(),
            name: loaded.name.clone(),
            version: loaded.version.clone(),
            description: loaded.description.clone(),
            author: loaded.author.clone(),
            file_path: path.to_path_buf(),
            state: PluginState::Loaded,
            enabled_at: None,
            loaded_at: Utc::now(),
            config: serde_json::Value::Object(serde_json::Map::new()),
            error: None,
        };

        // Call on_plugin_load if the plugin has the function
        let mut loaded = loaded;
        if let Err(e) = loaded.call_on_load() {
            tracing::warn!("Plugin {} on_plugin_load failed: {:?}", plugin_id, e);
        }

        self.registry.register(metadata)?;
        self.loaded_plugins.insert(plugin_id.clone(), loaded);

        info!("Plugin loaded successfully: {}", plugin_id);
        Ok(())
    }

    /// Enable a plugin
    pub async fn enable_plugin(&self, id: &str) -> PluginResult<()> {
        let mut plugin = self
            .loaded_plugins
            .get_mut(id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;

        self.registry.update_state(id, PluginState::Enabled)?;

        // Call plugin's on_enable function if it exists
        if plugin.has_function("on_plugin_enable") {
            if let Err(e) = plugin.execute_hook("on_plugin_enable", &[]) {
                error!("Plugin {} on_enable failed: {:?}", id, e);
                // Still mark as enabled even if on_enable fails
            }
        }

        info!("Plugin enabled: {}", id);
        Ok(())
    }

    /// Disable a plugin
    pub async fn disable_plugin(&self, id: &str) -> PluginResult<()> {
        let mut plugin = self
            .loaded_plugins
            .get_mut(id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;

        self.registry.update_state(id, PluginState::Disabled)?;

        // Call plugin's on_disable function if it exists
        if plugin.has_function("on_plugin_disable") {
            if let Err(e) = plugin.execute_hook("on_plugin_disable", &[]) {
                error!("Plugin {} on_disable failed: {:?}", id, e);
            }
        }

        info!("Plugin disabled: {}", id);
        Ok(())
    }

    /// Unload a plugin
    pub async fn unload_plugin(&self, id: &str) -> PluginResult<()> {
        // Call on_plugin_unload if the plugin has the function
        if let Some(mut plugin) = self.loaded_plugins.get_mut(id) {
            if plugin.has_function("on_plugin_unload") {
                if let Err(e) = plugin.execute_hook("on_plugin_unload", &[]) {
                    tracing::warn!("Plugin {} on_unload failed: {:?}", id, e);
                }
            }
        }

        self.loaded_plugins.remove(id);
        self.registry.remove(id)?;
        info!("Plugin unloaded: {}", id);
        Ok(())
    }

    /// Get plugin metadata
    pub fn get_plugin(&self, id: &str) -> Option<PluginMetadata> {
        self.registry.get(id)
    }

    /// List all plugins
    pub fn list_plugins(&self) -> Vec<PluginMetadata> {
        self.registry.list()
    }

    /// Execute pre-message-send hooks
    pub async fn execute_pre_message_hooks(
        &self,
        ctx: MessageHookContext,
    ) -> PluginResult<HookResult> {
        for plugin_entry in self.loaded_plugins.iter() {
            let plugin_id = plugin_entry.key().clone();
            let metadata = self.registry.get(&plugin_id);

            // Only execute hooks from enabled plugins
            if metadata.map(|m| m.state) != Some(PluginState::Enabled) {
                continue;
            }

            // Get mutable access to the plugin
            if let Some(mut plugin) = self.loaded_plugins.get_mut(&plugin_id) {
                // Check if plugin has the hook function
                let func_name = "on_message_send";
                if !plugin.has_function(func_name) {
                    drop(plugin); // Release the mutable borrow before continuing
                    continue;
                }

                // Execute the hook
                match plugin.execute_hook(func_name, &ctx.msg_data) {
                    Ok(HookAction::Continue) => {}
                    Ok(HookAction::Stop(reason)) => {
                        return Ok(HookResult::Stop(reason));
                    }
                    Err(e) => {
                        tracing::error!(
                            plugin = %plugin_id,
                            "Hook execution error: {:?}",
                            e
                        );
                        // Continue with other plugins even if one fails
                    }
                }
            }
        }

        Ok(HookResult::Continue)
    }

    /// Execute post-message-send hooks
    pub async fn execute_post_message_hooks(&self, msg_model: &entities::message_records::Model) {
        for plugin_entry in self.loaded_plugins.iter() {
            let plugin_id = plugin_entry.key().clone();
            let metadata = self.registry.get(&plugin_id);

            if metadata.map(|m| m.state) != Some(PluginState::Enabled) {
                continue;
            }

            // Get mutable access to the plugin
            if let Some(mut plugin) = self.loaded_plugins.get_mut(&plugin_id) {
                let func_name = "on_message_sent";
                if !plugin.has_function(func_name) {
                    drop(plugin); // Release the mutable borrow
                    continue;
                }

                // Serialize msg_id to bytes
                let msg_id_bytes = msg_model.msg_id.to_le_bytes().to_vec();

                if let Err(e) = plugin.execute_hook(func_name, &msg_id_bytes) {
                    tracing::error!(
                        plugin = %plugin_id,
                        msg_id = msg_model.msg_id,
                        "Post-hook execution error: {:?}",
                        e
                    );
                }
            }
        }
    }

    /// Check if plugin system is enabled
    pub fn is_enabled(&self) -> bool {
        !self.loaded_plugins.is_empty()
    }
}
