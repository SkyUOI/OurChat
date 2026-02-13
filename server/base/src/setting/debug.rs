use crate::constants;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DebugCfg {
    #[serde(default = "constants::default_debug_console")]
    pub debug_console: bool,
    #[serde(default = "constants::default_debug_console_port")]
    pub debug_console_port: u16,
}

impl Default for DebugCfg {
    fn default() -> Self {
        let empty = serde_json::json!({});
        serde_json::from_value(empty).unwrap()
    }
}
