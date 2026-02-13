#![feature(decl_macro)]
#![feature(duration_constructors)]

pub mod constants;
pub mod database;
pub mod email_client;
pub mod log;
pub mod rabbitmq;
pub mod setting;
pub mod shutdown;
pub mod types;
pub mod wrapper;

shadow_rs::shadow!(build);

pub const fn version_display() -> &'static str {
    const TAG: &str = if cfg!(feature = "official") {
        "official"
    } else {
        "custom"
    };
    const_format::formatcp!(
        r#"
OurChat Version: {}.{}
Commit Hash: {}
Build Time: {}
Build Env: {}, {}"#,
        build::PKG_VERSION,
        TAG,
        build::SHORT_COMMIT,
        build::BUILD_TIME,
        build::RUST_VERSION,
        build::RUST_CHANNEL
    )
}
