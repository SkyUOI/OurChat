//! Shared state of server
//! TODO: remove all of these and add test cases

use base::consts::{self};
use parking_lot::Mutex;

static FRIENDS_NUMBER_LIMIT: Mutex<u32> = Mutex::new(consts::default_friends_number_limit());

pub fn set_friends_number_limit(limit: u32) {
    *FRIENDS_NUMBER_LIMIT.lock() = limit;
}

pub fn get_friends_number_limit() -> u32 {
    *FRIENDS_NUMBER_LIMIT.lock()
}
