use crate::consts;
use parking_lot::Mutex;

pub static AUTO_CLEAN_DURATION: Mutex<u64> = Mutex::new(consts::default_clear_interval());
