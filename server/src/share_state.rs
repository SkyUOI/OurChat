use crate::consts;
use parking_lot::Mutex;
use static_keys::{define_static_key_false, static_branch_unlikely};

static AUTO_CLEAN_DURATION: Mutex<u64> = Mutex::new(consts::default_clear_interval());
static FILE_SAVE_DAYS: Mutex<u64> = Mutex::new(consts::default_file_save_days());

#[inline]
pub fn get_auto_clean_duration() -> u64 {
    *AUTO_CLEAN_DURATION.lock()
}

#[inline]
pub fn set_auto_clean_duration(duration: u64) {
    *AUTO_CLEAN_DURATION.lock() = duration;
    tracing::info!("set auto_clean_duration: {}", duration);
}

#[inline]
pub fn get_file_save_days() -> u64 {
    *FILE_SAVE_DAYS.lock()
}

#[inline]
pub fn set_file_save_days(days: u64) {
    *FILE_SAVE_DAYS.lock() = days;
    tracing::info!("set file_save_days: {}", days);
}

define_static_key_false!(MAINTAINING);

#[inline]
pub fn get_maintaining() -> bool {
    static_branch_unlikely!(MAINTAINING)
}

#[inline]
pub unsafe fn set_maintaining(maintaining: bool) {
    unsafe {
        if maintaining {
            MAINTAINING.enable();
        } else {
            MAINTAINING.disable();
        }
        tracing::info!("set maintaining: {}", maintaining);
    }
}
