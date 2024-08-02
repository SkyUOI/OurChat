//! Some utils functions
use crate::{consts, MACHINE_ID};
use rand::seq::SliceRandom;
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
pub static WS_BIND_ADDR: LazyLock<String> =
    LazyLock::new(|| format!("ws://{}:{}", consts::DEFAULT_IP, consts::DEFAULT_PORT));
