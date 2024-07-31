//! Some utils functions
use crate::{consts, machine_id};
use rand::seq::SliceRandom;
use snowdon::{ClassicLayout, Epoch, Generator, MachineId, Snowflake};
use std::sync::OnceLock;

pub struct SnowflakeParams;

impl Epoch for SnowflakeParams {
    fn millis_since_unix() -> u64 {
        1288834974657
    }
}

impl MachineId for SnowflakeParams {
    fn machine_id() -> u64 {
        machine_id()
    }
}

pub type MySnowflake = Snowflake<ClassicLayout<SnowflakeParams>, SnowflakeParams>;
pub type MySnowflakeGenerator = Generator<ClassicLayout<SnowflakeParams>, SnowflakeParams>;

/// 生成一个Snowflake的生成器
pub fn generator() -> &'static MySnowflakeGenerator {
    static GENERATOR: OnceLock<MySnowflakeGenerator> = OnceLock::new();
    GENERATOR.get_or_init(MySnowflakeGenerator::default)
}

const RANDOM_CHAR_POOL: [char; 62] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B',
    'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U',
    'V', 'W', 'X', 'Y', 'Z',
];

/// 生成一个随机的ocid
pub fn generate_ocid(bits: u32) -> String {
    let mut ret = String::new();
    ret.reserve(bits as usize);
    let mut rng = rand::thread_rng();
    for _ in 0..bits {
        let c = RANDOM_CHAR_POOL.choose(&mut rng).unwrap();
        ret.push(*c)
    }
    ret
}

/// 获取ws绑定地址
pub fn ws_bind_addr() -> &'static str {
    static BIND_ADDR: OnceLock<String> = OnceLock::new();
    BIND_ADDR.get_or_init(|| format!("ws://{}:{}", consts::DEFAULT_IP, consts::DEFAULT_PORT))
}
