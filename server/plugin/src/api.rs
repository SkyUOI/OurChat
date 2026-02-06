//! Host API exposed to WASM plugins
//!
//! Defines functions that plugins can call to interact with the host system

use crate::error::PluginError;
use crate::error::PluginResult;
use crate::PluginContext;

/// Host function implementation for ourchat_log (legacy)
fn ourchat_log_impl(
    mut caller: wasmtime::Caller<'_, PluginContext>,
    level: i32,
    ptr: i32,
    len: i32,
) {
    let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
        Some(m) => m,
        None => {
            tracing::error!(
                plugin = %caller.data().state.plugin_id,
                "ourchat_log failed: failed to find memory export"
            );
            return;
        }
    };

    let memory_size = memory.data_size(&caller);
    tracing::debug!(
        plugin = %caller.data().state.plugin_id,
        "ourchat_log called: level={}, ptr={}, len={}, memory_size={}",
        level, ptr, len, memory_size
    );

    if ptr < 0 || len < 0 {
        tracing::error!(
            plugin = %caller.data().state.plugin_id,
            "ourchat_log invalid parameters: ptr={}, len={}",
            ptr, len
        );
        return;
    }

    let ptr_usize = ptr as usize;
    let len_usize = len as usize;

    if ptr_usize + len_usize > memory_size {
        tracing::error!(
            plugin = %caller.data().state.plugin_id,
            "ourchat_log read out of bounds: ptr={}, len={}, memory_size={}",
            ptr, len, memory_size
        );
        return;
    }

    let mut data = vec![0u8; len_usize];
    match memory.read(&caller, ptr_usize, &mut data) {
        Ok(_) => {}
        Err(e) => {
            tracing::error!(
                plugin = %caller.data().state.plugin_id,
                "ourchat_log memory read failed: {}",
                e
            );
            return;
        }
    }

    let msg = String::from_utf8_lossy(&data);
    let plugin_id = &caller.data().state.plugin_id;

    match level {
        0 => tracing::trace!(plugin = %plugin_id, "{}", msg),
        1 => tracing::debug!(plugin = %plugin_id, "{}", msg),
        2 => tracing::info!(plugin = %plugin_id, "{}", msg),
        3 => tracing::warn!(plugin = %plugin_id, "{}", msg),
        4 => tracing::error!(plugin = %plugin_id, "{}", msg),
        _ => tracing::info!(plugin = %plugin_id, "{}", msg),
    }
}

/// Register all host functions with the linker
pub fn register_host_functions(
    linker: &mut wasmtime::Linker<PluginContext>,
) -> PluginResult<()> {
    // Logging function - must match WASM signature exactly
    linker
        .func_wrap(
            "env",
            "ourchat_log",
            |mut caller: wasmtime::Caller<'_, PluginContext>, level: i32, ptr: i32, len: i32| {
                let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                    Some(m) => m,
                    None => {
                        tracing::error!(
                            plugin = %caller.data().state.plugin_id,
                            "ourchat_log failed: failed to find memory export"
                        );
                        return;
                    }
                };

                let memory_size = memory.data_size(&caller);
                tracing::debug!(
                    plugin = %caller.data().state.plugin_id,
                    "ourchat_log called: level={}, ptr={}, len={}, memory_size={}",
                    level, ptr, len, memory_size
                );

                if ptr < 0 || len < 0 || len > 4096 {
                    tracing::error!(
                        plugin = %caller.data().state.plugin_id,
                        "ourchat_log invalid parameters: ptr={}, len={}",
                        ptr, len
                    );
                    return;
                }

                let ptr_usize = ptr as usize;
                let len_usize = len as usize;

                if ptr_usize + len_usize > memory_size || ptr_usize + len_usize > 1048832 {
                    tracing::error!(
                        plugin = %caller.data().state.plugin_id,
                        "ourchat_log read out of bounds: ptr={}, len={}, memory_size={}",
                        ptr, len, memory_size
                    );
                    return;
                }

                let mut data = vec![0u8; len_usize];
                match memory.read(&caller, ptr_usize, &mut data) {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!(
                            plugin = %caller.data().state.plugin_id,
                            "ourchat_log memory read failed: {}",
                            e
                        );
                        return;
                    }
                }

                let msg = String::from_utf8_lossy(&data);
                let plugin_id = &caller.data().state.plugin_id;

                match level {
                    0 => tracing::trace!(plugin = %plugin_id, "{}", msg),
                    1 => tracing::debug!(plugin = %plugin_id, "{}", msg),
                    2 => tracing::info!(plugin = %plugin_id, "{}", msg),
                    3 => tracing::warn!(plugin = %plugin_id, "{}", msg),
                    4 => tracing::error!(plugin = %plugin_id, "{}", msg),
                    _ => tracing::info!(plugin = %plugin_id, "{}", msg),
                }
            },
        )
        .map_err(|e| PluginError::Wasm(e.to_string()))?;

    Ok(())
}

/// Register component functions with the component linker
/// This is a placeholder for the WIT-based component function registration
pub fn register_component_functions(
    _linker: &mut wasmtime::component::Linker<PluginContext>,
) -> PluginResult<()> {
    // TODO: Once WIT bindings are generated, this will register the host implementations
    // For now, this is a placeholder
    tracing::warn!("Component function registration not yet implemented - WIT bindings need to be generated");
    Ok(())
}

