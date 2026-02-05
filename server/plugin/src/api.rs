//! Host API exposed to WASM plugins
//!
//! Defines functions that plugins can call to interact with the host system

use crate::error::PluginError;
use crate::error::PluginResult;
use crate::PluginContext;

/// Register all host functions with the linker
pub fn register_host_functions(
    linker: &mut wasmtime::Linker<PluginContext>,
) -> PluginResult<()> {
    // Logging function
    linker
        .func_wrap(
            "ourchat",
            "log",
            |mut caller: wasmtime::Caller<'_, PluginContext>,
             level: u32,
             ptr: u32,
             len: u32|
             -> Result<(), wasmtime::Error> {
                let memory = caller
                    .get_export("memory")
                    .and_then(|e| e.into_memory())
                    .ok_or_else(|| wasmtime::Error::msg("failed to find memory export"))?;

                let mut data = vec![0u8; len as usize];
                memory.read(&caller, ptr as usize, &mut data)?;

                let msg = String::from_utf8_lossy(&data);
                let plugin_id = &caller.data().plugin_id;

                match level {
                    0 => tracing::trace!(plugin = %plugin_id, "{}", msg),
                    1 => tracing::debug!(plugin = %plugin_id, "{}", msg),
                    2 => tracing::info!(plugin = %plugin_id, "{}", msg),
                    3 => tracing::warn!(plugin = %plugin_id, "{}", msg),
                    4 => tracing::error!(plugin = %plugin_id, "{}", msg),
                    _ => tracing::info!(plugin = %plugin_id, "{}", msg),
                }

                Ok(())
            },
        )
        .map_err(|e| PluginError::Wasm(e.to_string()))?;

    // Plugin config - get
    linker
        .func_wrap(
            "ourchat",
            "get_config",
            |mut caller: wasmtime::Caller<'_, PluginContext>,
             key_ptr: u32,
             key_len: u32|
             -> Result<(), wasmtime::Error> {
                let memory = caller
                    .get_export("memory")
                    .and_then(|e| e.into_memory())
                    .ok_or_else(|| wasmtime::Error::msg("failed to find memory export"))?;

                let mut key_data = vec![0u8; key_len as usize];
                memory.read(&caller, key_ptr as usize, &mut key_data)?;
                let key = String::from_utf8_lossy(&key_data);

                let config = caller.data().config.read();
                // Convert String key to &str for indexing
                let _value = config.get(key.as_ref());

                // TODO: Return value to WASM - needs more complex handling
                tracing::debug!("Plugin requested config: {}", key);

                Ok(())
            },
        )
        .map_err(|e| PluginError::Wasm(e.to_string()))?;

    Ok(())
}
