use base::{constants::CONFIG_FILE_ENV_VAR, setting::read_config_and_deserialize};
use serde::Deserialize;
use server::get_configuration;
use std::{
    fs::{self, File},
    path::PathBuf,
};

#[tokio::test]
async fn test_merge_config() -> anyhow::Result<()> {
    // Create temporary config files
    let temp_dir = tempfile::tempdir()?;
    let base_config_path: PathBuf = if let Ok(env) = std::env::var(CONFIG_FILE_ENV_VAR) {
        env
    } else {
        panic!("Please specify config file in .env file");
    }
    .into();
    let override_config_path = temp_dir.path().join("override.json");

    // Override configuration
    let override_config = serde_json::json!({
        "user_files_limit": "100MiB",
        "password_hash": {
            "m_cost": 4096
        },
        "debug": {
            "log_level": "debug"
        }
    });

    // Write config files
    serde_json::to_writer(File::create(&override_config_path)?, &override_config)?;

    // Create required files
    let required_files = [
        "redis.json",
        "db.json",
        "rabbitmq.json",
        "user_setting.json",
    ];
    for file in required_files.iter() {
        let file_path = temp_dir.path().join(file);
        fs::write(&file_path, "{}")?;
    }

    // Test config merging
    let config = get_configuration(vec![base_config_path.clone(), override_config_path.clone()])?;

    // Verify merge results
    assert_eq!(
        config.main_cfg.user_files_limit,
        base::constants::default_user_files_store_limit()
    ); // Should use override value
    assert_eq!(config.main_cfg.password_hash.m_cost, 4096); // Should use override value
    assert_eq!(config.main_cfg.password_hash.t_cost, 2); // Should keep base value
    assert_eq!(config.main_cfg.password_hash.p_cost, 1); // Should keep base value

    // Test config file paths
    assert_eq!(config.main_cfg.cmd_args.config.len(), 2);
    assert_eq!(config.main_cfg.cmd_args.config[0], base_config_path);
    assert_eq!(config.main_cfg.cmd_args.config[1], override_config_path);

    Ok(())
}

#[tokio::test]
async fn test_read_config_and_deserialize() -> anyhow::Result<()> {
    // Create temporary config files
    let temp_dir = tempfile::tempdir()?;
    let original_config_path = temp_dir.path().join("original_config.json");
    let new_config_path = temp_dir.path().join("new_config.json");

    #[derive(Deserialize)]
    struct TestConfig {
        ip: String,
        port: u16,
    }

    // Override configuration
    let override_config = serde_json::json! ({
        "ip": "127.0.0.1",
        "port": 8090,
    });

    let new_config = serde_json::json! ({
        "inherit": "original_config.json",
        "port": 9090,
    });
    // Write config files
    serde_json::to_writer(File::create(&original_config_path)?, &override_config)?;
    serde_json::to_writer(File::create(&new_config_path)?, &new_config)?;

    let config: TestConfig = read_config_and_deserialize(new_config_path).unwrap();

    assert_eq!(config.port, 9090);
    assert_eq!(config.ip, "127.0.0.1");

    Ok(())
}
