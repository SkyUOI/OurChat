//! OurChat Plugin System
//!
//! A secure, sandboxed plugin system using WebAssembly.

pub mod engine;
pub mod manager;
pub mod hooks;
pub mod api;
pub mod registry;
pub mod error;

pub use engine::{WasmEngine, PluginContext, LoadedPlugin};
pub use manager::PluginManager;
pub use hooks::{HookRegistry, Hook, MessageHookContext};
pub use error::{PluginError, PluginResult};
