mod helper;
mod http;
mod test_login_register;
mod test_status;
mod test_upload;
mod test_verify;

use libc_print::libc_println;
use std::{fs::read_dir, path::PathBuf};

fn delete_sqlite_db(mut path: PathBuf) {
    libc_println!("delete sqlite db: {:?}", path);
    std::fs::remove_file(&path).ok();
    path.set_extension("db-shm");
    libc_println!("delete sqlite db-shm {:?}", path);
    std::fs::remove_file(&path).ok();
    path.set_extension("db-wal");
    libc_println!("delete sqlite db-wal {:?}", path);
    std::fs::remove_file(&path).ok();
}

#[ctor::ctor]
fn cleanup_sqlite_database() {
    for i in read_dir(".").unwrap() {
        let path = i.unwrap().path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "db" {
                    delete_sqlite_db(path);
                }
            }
        }
    }
}
