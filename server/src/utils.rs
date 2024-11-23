//! Some utils functions
use crate::{
    SERVER_INFO,
    consts::{self, SessionID},
};
use rand::Rng;
use snowdon::{
    ClassicLayout, ClassicLayoutSnowflakeExtension, Epoch, Generator, MachineId, Snowflake,
};
use std::sync::LazyLock;
use tokio::task::JoinHandle;

pub struct SnowflakeParams;

impl Epoch for SnowflakeParams {
    fn millis_since_unix() -> u64 {
        1288834974657
    }
}

impl MachineId for SnowflakeParams {
    fn machine_id() -> u64 {
        SERVER_INFO.machine_id
    }
}

pub type MySnowflake = Snowflake<ClassicLayout<SnowflakeParams>, SnowflakeParams>;
pub type MySnowflakeGenerator = Generator<ClassicLayout<SnowflakeParams>, SnowflakeParams>;

/// A Generator of Snowflake
pub static GENERATOR: LazyLock<MySnowflakeGenerator> = LazyLock::new(MySnowflakeGenerator::default);

/// Generate ocid by random
pub fn generate_ocid(bits: usize) -> String {
    generate_random_string(bits)
}

pub fn generate_random_string(len: usize) -> String {
    let rng = rand::thread_rng();
    rng.sample_iter(rand::distributions::Alphanumeric)
        .map(char::from)
        .take(len)
        .collect()
}

/// default ws binging address
pub static DEFAULT_WS_BIND_ADDR: LazyLock<String> =
    LazyLock::new(|| gen_ws_bind_addr(consts::DEFAULT_IP, consts::DEFAULT_PORT));

pub fn gen_ws_bind_addr(ip: &str, port: u16) -> String {
    format!("ws://{}:{}", ip, port)
}

pub fn error_chain(e: anyhow::Error) -> String {
    let mut msg = String::new();
    for i in e.chain() {
        msg = format!("{msg}\nCaused by {}", i.to_string().as_str());
    }
    msg
}

pub fn spawn_blocking_with_tracing<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || current_span.in_scope(f))
}

pub fn generate_session_id() -> anyhow::Result<SessionID> {
    Ok(GENERATOR.generate()?.into_i64().into())
}

pub fn from_google_timestamp(ts: &prost_types::Timestamp) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::from_timestamp(ts.seconds, ts.nanos as u32)
}

pub fn to_google_timestamp(ts: chrono::DateTime<chrono::Utc>) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: ts.timestamp(),
        nanos: ts.timestamp_subsec_nanos() as i32,
    }
}

pub fn get_available_port() -> u16 {
    std::net::TcpListener::bind("0.0.0.0:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_string() {
        let s = generate_random_string(10);
        assert_eq!(s.len(), 10);
    }

    #[test]
    fn test_generate_ocid() {
        let s = generate_ocid(10);
        assert_eq!(s.len(), 10);
    }

    #[test]
    fn test_gen_ws_bind_addr() {
        let s = gen_ws_bind_addr("127.0.0.1", 8080);
        assert_eq!(s, "ws://127.0.0.1:8080");
    }
}
