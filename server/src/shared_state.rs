use crate::consts::{self, FileSize};
use parking_lot::Mutex;
use static_keys::{define_static_key_false, static_branch_unlikely};

static AUTO_CLEAN_DURATION: Mutex<u64> = Mutex::new(consts::default_clear_interval());
static FILE_SAVE_DAYS: Mutex<u64> = Mutex::new(consts::default_file_save_days());
static USER_FILES_STORE_LIMIT: Mutex<FileSize> =
    Mutex::new(consts::default_user_files_store_limit());
static FRIENDS_NUMBER_LIMIT: Mutex<u32> = Mutex::new(consts::default_friends_number_limit());

pub fn get_auto_clean_duration() -> u64 {
    *AUTO_CLEAN_DURATION.lock()
}

pub fn set_auto_clean_duration(duration: u64) {
    *AUTO_CLEAN_DURATION.lock() = duration;
    tracing::info!("set auto_clean_duration: {}", duration);
}

pub fn get_file_save_days() -> u64 {
    *FILE_SAVE_DAYS.lock()
}

pub fn set_file_save_days(days: u64) {
    *FILE_SAVE_DAYS.lock() = days;
    tracing::info!("set file_save_days: {}", days);
}

define_static_key_false!(MAINTAINING);

pub fn get_maintaining() -> bool {
    static_branch_unlikely!(MAINTAINING)
}

/// # Safety
/// should be called in multi-thread environment
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

pub fn get_user_files_store_limit() -> FileSize {
    *USER_FILES_STORE_LIMIT.lock()
}

pub fn set_user_files_store_limit(limit: FileSize) {
    *USER_FILES_STORE_LIMIT.lock() = limit;
    tracing::info!("set user_files_store_limit: {}", limit);
}

pub fn set_friends_number_limit(limit: u32) {
    *FRIENDS_NUMBER_LIMIT.lock() = limit;
}

pub fn get_friends_number_limit() -> u32 {
    *FRIENDS_NUMBER_LIMIT.lock()
}
