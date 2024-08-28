//! Some utils functions
use crate::{consts, MACHINE_ID};
use rand::{distributions::Alphanumeric, seq::SliceRandom, Rng};
use snowdon::{ClassicLayout, Epoch, Generator, MachineId, Snowflake};
use std::sync::LazyLock;

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
