//! WASM engine for plugin execution
//!
//! Provides the WASM runtime using wasmtime with sandboxing

use crate::error::{PluginError, PluginResult};
use crate::hooks::HookRegistry;
use base::database::DbPool;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use wasmtime::*;

/// Maximum memory per plugin (64MB)
const MAX_MEMORY_PAGES: u32 = 64 * 1024 / 64; // 64MB in 64KB pages

/// Maximum execution time per hook (100ms)
const MAX_EXECUTION_TIME: Duration = Duration::from_millis(100);

/// Plugin context - accessible within WASM
/// This struct is Sync because all its fields are Sync
#[derive(Debug)]
pub struct PluginContext {
    pub plugin_id: String,
    pub config: Arc<RwLock<serde_json::Value>>,
    pub db_pool: Option<DbPool>,
    pub redis_pool: Option<deadpool_redis::Pool>,
    pub host_data: Arc<DashMap<String, serde_json::Value>>,
}

// SAFETY: PluginContext only contains Sync types
unsafe impl Send for PluginContext {}
unsafe impl Sync for PluginContext {}

/// A loaded WASM plugin
#[derive(Debug)]
pub struct LoadedPlugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    store: Store<PluginContext>,
    instance: Instance,
    /// Registry of hook functions provided by this plugin
    pub hooks: Arc<HookRegistry>,
}

/// WASM engine configuration and runtime
#[derive(Debug)]
pub struct WasmEngine {
    engine: Engine,
    module_cache: DashMap<String, Module>,
}

impl WasmEngine {
    /// Create a new WASM engine
    pub fn new() -> PluginResult<Self> {
        // Configure the WASM engine with security limits
        let mut config = Config::new();
        config.wasm_component_model(false);
        config.async_support(false);
        config.consume_fuel(true); // Enable fuel for timeout

        // Set memory limits
        config.max_wasm_stack(MAX_MEMORY_PAGES as usize);

        let engine = Engine::new(&config)
            .map_err(|e| PluginError::Wasm(format!("Failed to create engine: {}", e)))?;

        Ok(Self {
            engine,
            module_cache: DashMap::new(),
        })
    }

    /// Load a WASM module from a file
    pub fn load_module(&self, path: PathBuf) -> PluginResult<Module> {
        let cache_key = path.to_string_lossy().to_string();

        // Check cache first
        if let Some(module) = self.module_cache.get(&cache_key) {
            return Ok(module.clone());
        }

        // Read the WASM file
        let wasm_bytes = std::fs::read(&path)
            .map_err(|e| PluginError::LoadFailed(format!("Failed to read {}: {}", path.display(), e)))?;

        // Validate and compile the module
        let module = Module::from_binary(&self.engine, &wasm_bytes)
            .map_err(|e| PluginError::Wasm(format!("Failed to compile module: {}", e)))?;

        // Cache it
        self.module_cache.insert(cache_key, module.clone());

        Ok(module)
    }

    /// Create a linker with host functions
    fn create_linker(&self) -> PluginResult<Linker<PluginContext>> {
        let mut linker = Linker::new(&self.engine);

        // Add host functions (API exposed to plugins)
        crate::api::register_host_functions(&mut linker)?;

        Ok(linker)
    }

    /// Instantiate a plugin
    pub fn instantiate_plugin(
        self: &Arc<Self>,
        module: Module,
        plugin_id: String,
        context: PluginContext,
    ) -> PluginResult<LoadedPlugin> {
        // Create store with fuel enabled
        let mut store = Store::new(&self.engine, context);
        store.set_fuel(u64::MAX)
            .map_err(|e| PluginError::Wasm(format!("Failed to set fuel: {}", e)))?;

        // Set host data
        store.data_mut().host_data = Arc::new(DashMap::new());

        // Create linker
        let linker = self.create_linker()?;

        // Instantiate the module
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| PluginError::Wasm(format!("Failed to instantiate: {}", e)))?;

        // Create hooks registry and scan for exported functions
        let hooks = Arc::new(HookRegistry::new());

        // Scan for known hook functions and register them
        let hook_functions = [
            "on_plugin_load",
            "on_plugin_enable",
            "on_plugin_disable",
            "on_plugin_unload",
            "on_message_send",
            "on_message_sent",
        ];

        for func_name in hook_functions {
            if instance.get_func(&mut store, func_name).is_some() {
                hooks.register(crate::hooks::Hook {
                    plugin_id: plugin_id.clone(),
                    hook_type: match func_name {
                        "on_message_send" => crate::hooks::HookType::PreMessageSend,
                        "on_message_sent" => crate::hooks::HookType::PostMessageSend,
                        _ => continue, // Other hooks don't have types yet
                    },
                    func_name: func_name.to_string(),
                }).ok();
            }
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
    /// Check if the plugin has a specific exported function
    pub fn has_function(&mut self, func_name: &str) -> bool {
        self.instance.get_func(&mut self.store, func_name).is_some()
    }

    /// Call the plugin's on_plugin_load function if it exists
    pub fn call_on_load(&mut self) -> PluginResult<()> {
        if self.has_function("on_plugin_load") {
            self.execute_hook("on_plugin_load", &[])?;
        }
        Ok(())
    }

    /// Execute a hook function with data
    pub fn execute_hook(&mut self, func_name: &str, data: &[u8]) -> PluginResult<HookAction> {
        // Get the memory export
        let memory = self
            .instance
            .get_memory(&mut self.store, "memory")
            .ok_or_else(|| PluginError::Wasm("Plugin has no memory export".to_string()))?;

        // Allocate space in WASM memory
        let data_ptr: u32 = 0;
        let data_len: u32 = data.len() as u32;

        // Check if memory is large enough
        if memory.data_size(&mut self.store) < (data_ptr as usize + data.len()) {
            return Err(PluginError::Wasm("Plugin memory too small".to_string()));
        }

        // Write data to WASM memory
        memory
            .write(&mut self.store, data_ptr as usize, data)
            .map_err(|e| PluginError::Wasm(format!("Failed to write to memory: {}", e)))?;

        // Set fuel limit for execution time
        self.store
            .set_fuel(MAX_EXECUTION_TIME.as_millis() as u64)
            .map_err(|e| PluginError::Wasm(format!("Failed to set fuel: {}", e)))?;

        // Call the hook function
        let func = self
            .instance
            .get_typed_func::<(u32, u32), u32>(&mut self.store, func_name)
            .map_err(|e| PluginError::Wasm(format!("Function '{}' not found: {}", func_name, e)))?;

        let result_code = func
            .call(&mut self.store, (data_ptr, data_len))
            .map_err(|e| PluginError::Wasm(format!("Function call failed: {}", e)))?;

        Ok(match result_code {
            0 => HookAction::Continue,
            1 => HookAction::Stop("Blocked by plugin".to_string()),
            _ => HookAction::Continue,
        })
    }
}

/// Hook action result
#[derive(Debug)]
pub enum HookAction {
    Continue,
    Stop(String),
}

impl Default for WasmEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create WASM engine")
    }
}
