//! Host implementations for WIT interfaces
//!
//! This module provides the host-side implementations of the WIT interfaces
//! that are exposed to plugins.

use crate::engine::PluginContext;
use crate::error::PluginResult;
use wasmtime::component::Linker;

// Import the generated WIT bindings
// The bindings generate types and functions for interacting with guests
use crate::bindings::exports::ourchat::plugin;

/// Register all host implementations with the component linker
///
/// This includes:
/// - WASI preview2 implementations (required for Rust plugins)
/// - Any custom host functions
///
/// For the WIT world defined in ourchat.wit:
/// - The guest (plugin) exports: hooks, plugin-lifecycle
/// - The guest imports: logging, config (from host via WASI)
/// - The host also provides WASI implementations for I/O
pub fn register_host_implementations(
    linker: &mut Linker<PluginContext>,
) -> PluginResult<()> {
    // Add WASI preview2 support to the linker
    // This registers all WASI interfaces (cli, clocks, filesystem, random, sockets, etc.)
    wasmtime_wasi::p2::add_to_linker_sync(linker)
        .map_err(|e| crate::error::PluginError::Wasm(format!("Failed to add WASI to linker: {}", e)))?;

    Ok(())
}

/// Wrapper for accessing plugin exports through WIT bindings
///
/// This provides a type-safe interface to call the plugin's exported functions.
pub struct PluginExports {
    // The actual exports will be accessed through the generated bindings
    // This is a placeholder for the wrapper structure
    _marker: std::marker::PhantomData<()>,
}

impl PluginExports {
    /// Create a new exports wrapper from a component instance
    pub fn new(
        _instance: &wasmtime::component::Instance,
        _store: &mut wasmtime::Store<PluginContext>,
    ) -> PluginResult<Self> {
        // TODO: Use the generated bindings to wrap the instance
        // The generated code will provide methods to call the plugin's exports
        Ok(Self {
            _marker: std::marker::PhantomData,
        })
    }

    /// Call the plugin's on_load lifecycle function
    pub fn call_on_load(&mut self) -> PluginResult<Result<(), String>> {
        // TODO: Use the generated bindings to call the export
        // This will look something like:
        // plugin_lifecycle::on_load(store)
        tracing::warn!("call_on_load not yet implemented");
        Ok(Ok(()))
    }

    /// Call the plugin's on_message_send hook
    pub fn call_on_message_send(
        &mut self,
        _ctx: plugin::hooks::MessageContext,
    ) -> PluginResult<plugin::hooks::HookResult> {
        // TODO: Use the generated bindings to call the export
        // This will look something like:
        // hooks::on_message_send(store, ctx)
        tracing::warn!("call_on_message_send not yet implemented");
        Ok(plugin::hooks::HookResult::Continue)
    }
}
