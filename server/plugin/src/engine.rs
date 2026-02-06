//! WASM engine for plugin execution
//!
//! Provides the WASM runtime using wasmtime component model with sandboxing

use crate::error::{PluginError, PluginResult};
use crate::hooks::HookRegistry;
use base::database::DbPool;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;
use wasmtime::{Engine, Config, Module, Linker as WasmtimeLinker, Store};
use wasmtime::component::{Component, Linker as ComponentLinker, Instance as ComponentInstance};

// For now, we'll use a simpler approach without full WASI preview2 integration
// Once the WIT bindings are generated, we'll properly integrate WASI

/// Maximum memory per plugin (64MB)
const MAX_MEMORY_PAGES: u32 = 64 * 1024 / 64; // 64MB in 64KB pages

/// Plugin state - shared state for component model
#[derive(Clone, Debug)]
pub struct PluginState {
    pub plugin_id: String,
    pub config: Arc<RwLock<serde_json::Value>>,
    pub db_pool: Option<DbPool>,
    pub redis_pool: Option<deadpool_redis::Pool>,
    pub host_data: Arc<DashMap<String, serde_json::Value>>,
}

/// Plugin context - accessible within WASM components
/// Simplified version without full WASI integration for now
pub struct PluginContext {
    pub state: PluginState,
}

/// A loaded WASM component plugin
pub struct LoadedPlugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    store: Store<PluginContext>,
    instance: ComponentInstance,
    /// Registry of hook functions provided by this plugin
    pub hooks: Arc<HookRegistry>,
}

/// WASM engine configuration and runtime for component model
pub struct WasmEngine {
    engine: Engine,
    component_cache: DashMap<String, Component>,
}

impl WasmEngine {
    /// Create a new WASM engine with component model support
    pub fn new() -> PluginResult<Self> {
        // Configure the WASM engine with security limits and component model
        let mut config = Config::new();

        // Enable component model
        config.wasm_component_model(true);

        // Set memory limits
        config.max_wasm_stack(MAX_MEMORY_PAGES as usize);

        // Enable async support
        config.async_support(true);

        let engine = Engine::new(&config)
            .map_err(|e| PluginError::Wasm(format!("Failed to create engine: {}", e)))?;

        Ok(Self {
            engine,
            component_cache: DashMap::new(),
        })
    }

    /// Load a WASM component from a file
    pub fn load_component(&self, path: PathBuf) -> PluginResult<Component> {
        let cache_key = path.to_string_lossy().to_string();

        // Check cache first
        if let Some(component) = self.component_cache.get(&cache_key) {
            return Ok(component.clone());
        }

        // Read the WASM component file
        let wasm_bytes = std::fs::read(&path)
            .map_err(|e| PluginError::LoadFailed(format!("Failed to read {}: {}", path.display(), e)))?;

        // Validate and compile the component
        let component = Component::from_binary(&self.engine, &wasm_bytes)
            .map_err(|e| PluginError::Wasm(format!("Failed to compile component: {}", e)))?;

        // Cache it
        self.component_cache.insert(cache_key, component.clone());

        Ok(component)
    }

    /// Load legacy WASM module and adapt to component
    pub fn load_module_as_component(&self, path: PathBuf) -> PluginResult<Component> {
        // Read the legacy WASM module
        let wasm_bytes = std::fs::read(&path)
            .map_err(|e| PluginError::LoadFailed(format!("Failed to read {}: {}", path.display(), e)))?;

        // Use the component adapter to convert legacy module to component
        // This is a placeholder - actual implementation would use wasm-tools
        tracing::warn!("Loading legacy WASM module - conversion to component not yet implemented");

        // For now, try loading as component directly
        Component::from_binary(&self.engine, &wasm_bytes)
            .map_err(|e| PluginError::Wasm(format!("Failed to load as component: {}", e)))
    }

    /// Create a component linker with host implementations
    fn create_component_linker(&self) -> PluginResult<ComponentLinker<PluginContext>> {
        let mut linker = ComponentLinker::new(&self.engine);

        // Register host implementations
        crate::host::register_host_implementations(&mut linker)?;

        Ok(linker)
    }

    /// Keep legacy linker for backward compatibility
    fn create_linker(&self) -> PluginResult<WasmtimeLinker<PluginContext>> {
        let mut linker = WasmtimeLinker::new(&self.engine);
        crate::api::register_host_functions(&mut linker)?;
        Ok(linker)
    }

    /// Keep legacy module loading for backward compatibility
    pub fn load_module(&self, path: PathBuf) -> PluginResult<Module> {
        let wasm_bytes = std::fs::read(&path)
            .map_err(|e| PluginError::LoadFailed(format!("Failed to read {}: {}", path.display(), e)))?;
        Module::from_binary(&self.engine, &wasm_bytes)
            .map_err(|e| PluginError::Wasm(format!("Failed to compile module: {}", e)))
    }

    /// Instantiate a component plugin
    pub fn instantiate_plugin(
        self: &Arc<Self>,
        component: Component,
        plugin_id: String,
        context: PluginContext,
    ) -> PluginResult<LoadedPlugin> {
        // Create store with component context
        let mut store = Store::new(&self.engine, context);

        // Create component linker
        let linker = self.create_component_linker()?;

        // Instantiate the component
        let instance = linker
            .instantiate(&mut store, &component)
            .map_err(|e| PluginError::Wasm(format!("Failed to instantiate component: {}", e)))?;

        // Create hooks registry
        let hooks = Arc::new(HookRegistry::new());

        // Register hooks - in component model, hooks are accessed via exports
        // For now, we'll register all hook types and let the manager call them
        let hook_types = [
            crate::hooks::HookType::PreMessageSend,
            crate::hooks::HookType::PostMessageSend,
            crate::hooks::HookType::UserCreated,
            crate::hooks::HookType::UserLogin,
            crate::hooks::HookType::FriendAdded,
            crate::hooks::HookType::SessionCreated,
        ];

        for hook_type in hook_types {
            hooks.register(crate::hooks::Hook {
                plugin_id: plugin_id.clone(),
                hook_type,
                func_name: format!("{:?}", hook_type),
            }).ok();
        }

        Ok(LoadedPlugin {
            id: plugin_id.clone(),
            name: plugin_id.clone(),
            version: "0.0.0".to_string(),
            description: String::new(),
            author: String::new(),
            store,
            instance,
            hooks,
        })
    }
}

impl LoadedPlugin {
    /// Check if the plugin has a specific exported function (legacy compatibility)
    pub fn has_function(&mut self, _func_name: &str) -> bool {
        // In component model, exports are accessed differently
        // For now, always return true to allow hook execution
        true
    }

    /// Call the plugin's on_plugin_load function if it exists
    pub fn call_on_load(&mut self) -> PluginResult<()> {
        // In component model, this would use the generated bindings
        // For now, we'll just log that the plugin was loaded
        tracing::info!("Plugin {} loaded (component model)", self.id);
        Ok(())
    }

    /// Execute a hook function (legacy compatibility)
    pub fn execute_hook(&mut self, func_name: &str, data: &[u8]) -> PluginResult<HookAction> {
        // In component model, hooks are called via the generated WIT bindings
        // For now, we'll return Continue to maintain compatibility
        tracing::debug!(
            "Executing hook '{}' for plugin {} (component model - not yet implemented)",
            func_name, self.id
        );
        Ok(HookAction::Continue)
    }

    /// Execute a hook using component model (new API)
    pub async fn execute_hook_component(
        &mut self,
        hook_type: crate::hooks::HookType,
        context: &crate::hooks::MessageHookContext,
    ) -> PluginResult<crate::hooks::HookResult> {
        // TODO: Implement component model hook execution
        // This will use the generated WIT bindings to call the plugin's hooks
        tracing::debug!(
            "Executing component hook {:?} for plugin {}",
            hook_type, self.id
        );
        Ok(crate::hooks::HookResult::Continue)
    }
}

/// Hook action result
#[derive(Debug)]
pub enum HookAction {
    Continue,
    Stop(String),
}
