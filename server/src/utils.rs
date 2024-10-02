//! Some utils functions
use crate::{MACHINE_ID, consts};
use rand::Rng;
use snowdon::{ClassicLayout, Epoch, Generator, MachineId, Snowflake};
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
        *MACHINE_ID
    }
}

pub type MySnowflake = Snowflake<ClassicLayout<SnowflakeParams>, SnowflakeParams>;
pub type MySnowflakeGenerator = Generator<ClassicLayout<SnowflakeParams>, SnowflakeParams>;

/// 一个Snowflake的生成器
pub static GENERATOR: LazyLock<MySnowflakeGenerator> = LazyLock::new(MySnowflakeGenerator::default);

/// 生成一个随机的ocid
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

/// 获取ws绑定地址
pub static WS_BIND_ADDR: LazyLock<String> =
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
    actix_web::rt::task::spawn_blocking(move || current_span.in_scope(f))
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
