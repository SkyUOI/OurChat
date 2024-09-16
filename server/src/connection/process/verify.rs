use crate::utils;

const TOKEN_LEN: usize = 20;

pub fn generate_token() -> String {
    utils::generate_random_string(TOKEN_LEN)
}
