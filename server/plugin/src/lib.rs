//! OurChat Plugin System
//!
//! A secure, sandboxed plugin system using WebAssembly with Component Model and WIT.

pub mod engine;
pub mod manager;
pub mod hooks;
pub mod api;
pub mod registry;
pub mod error;
pub mod host_impls;
pub mod host;
pub mod bindings;

pub use engine::{WasmEngine, PluginContext, LoadedPlugin};
pub use manager::PluginManager;
pub use hooks::{HookRegistry, Hook, MessageHookContext};
pub use error::{PluginError, PluginResult};
