//! Message Logger Plugin
//!
//! An example plugin that demonstrates the OurChat plugin SDK.

// Generate WIT bindings at the root level
wit_bindgen::generate!({
    path: "../../../plugin/wit/ourchat.wit",
});

// Use the SDK for logging helpers
use ourchat_plugin_sdk::logging;

// Import the types from the generated bindings
use exports::ourchat::plugin::hooks::{Guest as HooksGuest, HookResult, MessageContext, UserContext, SessionContext};
use exports::ourchat::plugin::plugin_lifecycle::Guest as PluginLifecycleGuest;

/// The message logger plugin
struct MessageLoggerPlugin;

// Implement the plugin lifecycle exports for the WIT interface
impl PluginLifecycleGuest for MessageLoggerPlugin {
    fn on_load() -> Result<(), String> {
        logging::info("Message Logger Plugin loaded");
        Ok(())
    }

    fn on_enable() -> Result<(), String> {
        logging::info("Message Logger Plugin enabled");
        Ok(())
    }

    fn on_disable() -> Result<(), String> {
        logging::info("Message Logger Plugin disabled");
        Ok(())
    }

    fn on_unload() {
        logging::info("Message Logger Plugin unloaded");
    }
}

// Implement the hooks exports for the WIT interface
impl HooksGuest for MessageLoggerPlugin {
    fn on_message_send(ctx: MessageContext) -> HookResult {
        logging::info(&format!(
            "Message from {:?} in session {:?} (encrypted: {})",
            ctx.sender_id, ctx.session_id, ctx.is_encrypted
        ));

        // Try to parse the message data as JSON for better logging
        if let Ok(json_str) = String::from_utf8(ctx.msg_data.clone()) {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json_str) {
                logging::debug(&format!("Message content: {}", value));

                // Example: Block messages with specific content
                if let Some(obj) = value.as_object() {
                    if let Some(content) = obj.get("content").and_then(|v| v.as_str()) {
                        if content.contains("blocked") {
                            logging::warn("Message blocked by plugin");
                            return HookResult::Stop("Message contains blocked content".to_string());
                        }
                    }
                }
            }
        }

        HookResult::Continue
    }

    fn on_message_sent(msg_id: u64) {
        logging::info(&format!("Message {} sent successfully", msg_id));
    }

    fn on_user_created(ctx: UserContext) {
        logging::info(&format!(
            "New user created: {} ({})",
            ctx.username, ctx.email
        ));
    }

    fn on_user_login(ctx: UserContext) {
        logging::info(&format!("User logged in: {}", ctx.username));
    }

    fn on_friend_added(_user_id: u64, _friend_id: u64, _session_id: u64) {
        logging::info("Friend added");
    }

    fn on_session_created(ctx: SessionContext) {
        logging::info(&format!("New session created: {}", ctx.session_id));
    }
}

// Export the plugin implementation
export!(MessageLoggerPlugin);



