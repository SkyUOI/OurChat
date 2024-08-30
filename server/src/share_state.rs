use crate::consts;
use parking_lot::Mutex;

pub static AUTO_CLEAN_DURATION: Mutex<u64> = Mutex::new(consts::default_clear_interval());
pub static FILE_SAVE_DAYS: Mutex<u64> = Mutex::new(consts::default_file_save_days());
