//! Inter-plugin messaging host implementation
//!
//! Provides inter-plugin communication via the WIT messaging interface.

use crate::engine::PluginState;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::sync::Arc;

/// Plugin message
#[derive(Debug, Clone)]
pub struct PluginMessage {
    pub source_plugin: String,
    pub target_plugin: String,
    pub message_type: String,
    pub payload: Vec<u8>,
}

/// Messaging errors
#[derive(Debug)]
pub enum MessagingError {
    TargetNotFound,
    MessageTooLarge,
    SendFailed(String),
}

/// Message subscription
struct Subscription {
    plugin_id: String,
    message_type: String,
}

/// In-memory message bus for inter-plugin communication
struct MessageBus {
    subscriptions: DashMap<String, Vec<Subscription>>,
    max_message_size: usize,
}

impl MessageBus {
    fn new() -> Self {
        Self {
            subscriptions: DashMap::new(),
            max_message_size: 1024 * 1024, // 1MB default
        }
    }

    fn subscribe(&self, plugin_id: String, message_type: String) {
        let mut subs = self.subscriptions.entry(message_type.clone()).or_default();
        let sub = Subscription {
            plugin_id,
            message_type,
        };
        if !subs.iter().any(|s| s.plugin_id == sub.plugin_id) {
            subs.push(sub);
        }
    }

    fn unsubscribe(&self, plugin_id: &str, message_type: &str) {
        if let Some(mut subs) = self.subscriptions.get_mut(message_type) {
            subs.retain(|s| s.plugin_id != plugin_id);
        }
    }

    fn get_subscribers(&self, message_type: &str) -> Vec<String> {
        self.subscriptions
            .get(message_type)
            .map(|subs| subs.iter().map(|s| s.plugin_id.clone()).collect())
            .unwrap_or_default()
    }
}

/// Host implementation for the messaging interface
pub struct MessagingHost {
    state: Arc<RwLock<PluginState>>,
    bus: Arc<MessageBus>,
}

impl MessagingHost {
    pub fn new(state: Arc<RwLock<PluginState>>, bus: Arc<MessageBus>) -> Self {
        Self { state, bus }
    }
}

/// Implement the messaging interface from WIT
impl MessagingHost {
    pub fn send(&self, msg: PluginMessage) -> Result<(), MessagingError> {
        // Check message size
        if msg.payload.len() > self.bus.max_message_size {
            return Err(MessagingError::MessageTooLarge);
        }

        let state = self.state.read();
        tracing::debug!(
            plugin = %state.plugin_id,
            "Sending message from {} to {}: {}",
            msg.source_plugin, msg.target_plugin, msg.message_type
        );

        // TODO: Implement actual message delivery
        // This would involve finding the target plugin and delivering the message
        Ok(())
    }

    pub fn broadcast(&self, message_type: String, payload: Vec<u8>) {
        let state = self.state.read();
        tracing::debug!(
            plugin = %state.plugin_id,
            "Broadcasting message type {}: {} bytes",
            message_type,
            payload.len()
        );

        // Get subscribers
        let subscribers = self.bus.get_subscribers(&message_type);

        // TODO: Deliver message to all subscribers
        for subscriber in subscribers {
            tracing::debug!("Delivering to subscriber: {}", subscriber);
        }
    }

    pub fn subscribe(&self, message_type: String) {
        let plugin_id = self.state.read().plugin_id.clone();
        self.bus.subscribe(plugin_id, message_type);
        tracing::debug!("Plugin subscribed to message type");
    }

    pub fn unsubscribe(&self, message_type: String) {
        let plugin_id = self.state.read().plugin_id.clone();
        self.bus.unsubscribe(&plugin_id, &message_type);
        tracing::debug!("Plugin unsubscribed from message type");
    }
}
