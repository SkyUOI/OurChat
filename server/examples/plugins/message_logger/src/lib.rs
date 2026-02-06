//! Message Logger Plugin v2
//!
//! An example plugin that demonstrates the OurChat plugin SDK.

use ourchat_plugin_sdk::prelude::*;
use serde_json::Value as JsonValue;

/// The message logger plugin
struct MessageLoggerPlugin;

impl PluginLifecycle for MessageLoggerPlugin {
    fn on_load(&mut self) -> Result<(), String> {
        logging::info("🔧 Message Logger Plugin v2 loaded");
        Ok(())
    }

    fn on_enable(&mut self) -> Result<(), String> {
        logging::info("✅ Message Logger Plugin v2 enabled");
        Ok(())
    }

    fn on_disable(&mut self) -> Result<(), String> {
        logging::info("⏸️  Message Logger Plugin v2 disabled");
        Ok(())
    }

    fn on_unload(&mut self) {
        logging::info("👋 Message Logger Plugin v2 unloaded");
    }
}

impl MessageHooks for MessageLoggerPlugin {
    fn on_message_send(&mut self, ctx: &MessageContext) -> HookResult {
        logging::info(&format!(
            "📨 Message from {:?} in session {:?} (encrypted: {})",
            ctx.sender_id, ctx.session_id, ctx.is_encrypted
        ));

        // Try to parse the message data as JSON for better logging
        if let Ok(json_str) = String::from_utf8(ctx.msg_data.clone()) {
            if let Ok(value) = serde_json::from_str::<JsonValue>(&json_str) {
                logging::debug(&format!("📄 Message content: {}", value));

                // Example: Block messages with specific content
                if let Some(obj) = value.as_object() {
                    if let Some(content) = obj.get("content").and_then(|v| v.as_str()) {
                        if content.contains("blocked") {
                            logging::warn("🚫 Message blocked by plugin");
                            return HookResult::Stop("Message contains blocked content".to_string());
                        }
                    }
                }
            }
        }

        HookResult::Continue
    }

    fn on_message_sent(&mut self, msg_id: u64) {
        logging::info(&format!("✉️  Message {} sent successfully", msg_id));
    }

    fn on_user_created(&mut self, ctx: &UserContext) {
        logging::info(&format!(
            "👤 New user created: {} ({})",
            ctx.username, ctx.email
        ));
    }

    fn on_user_login(&mut self, ctx: &UserContext) {
        logging::info(&format!("🔓 User logged in: {}", ctx.username));
    }

    fn on_session_created(&mut self, ctx: &SessionContext) {
        logging::info(&format!("💬 New session created: {}", ctx.session_id));
    }
}

// Plugin exports for the host
//
// NOTE: These are the legacy exports for backward compatibility.
// In the future, these will be generated from WIT bindings.

#[no_mangle]
pub extern "C" fn on_plugin_load() -> u32 {
    let mut plugin = MessageLoggerPlugin;
    if let Err(e) = plugin.on_load() {
        logging::error(&format!("Failed to load plugin: {}", e));
        return 1;
    }
    0
}

#[no_mangle]
pub extern "C" fn on_plugin_enable() -> u32 {
    let mut plugin = MessageLoggerPlugin;
    if let Err(e) = plugin.on_enable() {
        logging::error(&format!("Failed to enable plugin: {}", e));
        return 1;
    }
    0
}

#[no_mangle]
pub extern "C" fn on_plugin_disable() -> u32 {
    let mut plugin = MessageLoggerPlugin;
    if let Err(e) = plugin.on_disable() {
        logging::error(&format!("Failed to disable plugin: {}", e));
        return 1;
    }
    0
}

#[no_mangle]
pub extern "C" fn on_plugin_unload() -> u32 {
    let mut plugin = MessageLoggerPlugin;
    plugin.on_unload();
    0
}

#[no_mangle]
pub extern "C" fn on_message_send(ptr: u32, len: u32) -> u32 {
    // This would be implemented with proper memory access in a real plugin
    // For now, just return continue
    logging::debug("on_message_send called");
    0
}

#[no_mangle]
pub extern "C" fn on_message_sent(ptr: u32, len: u32) -> u32 {
    logging::debug("on_message_sent called");
    0
}
