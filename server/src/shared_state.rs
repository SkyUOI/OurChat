//! Shared state of server
//! TODO: remove all of these and add test cases

use base::consts::{self};
use parking_lot::Mutex;

static AUTO_CLEAN_DURATION: Mutex<u64> = Mutex::new(consts::default_clear_interval());
static FILE_SAVE_DAYS: Mutex<u64> = Mutex::new(consts::default_file_save_days());
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

pub fn set_friends_number_limit(limit: u32) {
    *FRIENDS_NUMBER_LIMIT.lock() = limit;
}

pub fn get_friends_number_limit() -> u32 {
    *FRIENDS_NUMBER_LIMIT.lock()
}
