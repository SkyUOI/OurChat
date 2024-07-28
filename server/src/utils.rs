use std::sync::OnceLock;

use snowdon::{
    ClassicLayout, ClassicLayoutSnowflakeExtension, Epoch, Generator, MachineId, Snowflake,
};

use crate::machine_id;

struct SnowflakeParams;

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

type MySnowflake = Snowflake<ClassicLayout<SnowflakeParams>, SnowflakeParams>;
type MySnowflakeGenerator = Generator<ClassicLayout<SnowflakeParams>, SnowflakeParams>;

fn generator() -> &'static MySnowflakeGenerator {
    static GENERATOR: OnceLock<MySnowflakeGenerator> = OnceLock::new();
    GENERATOR.get_or_init(|| MySnowflakeGenerator::default())
}
