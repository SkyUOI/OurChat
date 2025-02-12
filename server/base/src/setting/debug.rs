use crate::consts;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DebugCfg {
    #[serde(default = "consts::default_debug_console")]
    pub debug_console: bool,
    #[serde(default = "consts::default_debug_console_port")]
    pub debug_console_port: u16,
}
