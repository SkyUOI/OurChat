mod http;

use std::{path::PathBuf, time::Duration};

use anyhow::{Context, bail};
use base::constants::OCID;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use size::Size;
use utils::{merge_json, serde_default};

use crate::{ParserCfg, config::http::HttpCfg};
use base::{
    constants::{self, CONFIG_FILE_ENV_VAR, SessionID},
    database::{postgres::PostgresDbCfg, redis_cfg::RedisCfg},
    rabbitmq::RabbitMQCfg,
    setting::{self, PathConvert, Setting, UserSetting, debug::DebugCfg},
};

#[derive(Debug, Serialize, Clone, derive::PathConvert)]
pub struct MainCfg {
    pub inherit: Option<String>,
    pub redis_cfg: PathBuf,
    pub db_cfg: PathBuf,
    pub rabbitmq_cfg: PathBuf,
    pub user_setting: PathBuf,
    pub http_cfg: PathBuf,
    pub auto_clean_duration: croner::Cron,
    pub files_save_time: Duration,
    pub user_files_limit: Size,
    pub friends_number_limit: u32,
    pub files_storage_path: PathBuf,
    pub enable_file_cache: bool,
    pub enable_hierarchical_storage: bool,
    pub enable_file_deduplication: bool,
    pub enable_metrics: bool,
    pub metrics_snapshot_interval: Duration,
    pub cache_max_size: Size,
    pub verification_expire_time: Duration,
    pub user_defined_status_expire_time: Duration,
    pub log_clean_duration: Duration,
    pub log_keep: Duration,
    pub single_instance: bool,
    pub leader_node: bool,
    pub room_key_duration: Duration,
    pub unregister_policy: UnregisterPolicy,
    pub password_hash: PasswordHash,
    pub db: DbArgCfg,
    pub debug: DebugCfg,
    pub voip: VOIP,
    pub oauth: OAuthCfg,
    pub require_email_verification: bool,
    pub default_session: Option<SessionID>,
    pub lock_account_after_failed_logins: u32,
    pub lock_account_duration: Duration,
    pub initial_admin_ocid: Option<OCID>,

    #[serde(skip)]
    pub cmd_args: ParserCfg,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum UnregisterPolicy {
    #[default]
    Disable,
    Delete,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OAuthCfg {
    #[serde(default = "base::constants::default_oauth_enable")]
    pub enable: bool,
    #[serde(default = "base::constants::default_oauth_github_client_id")]
    pub github_client_id: String,
    #[serde(default = "base::constants::default_oauth_github_client_secret")]
    pub github_client_secret: String,
}

serde_default!(OAuthCfg);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PasswordHash {
    #[serde(default = "constants::default_m_cost")]
    pub m_cost: u32,
    #[serde(default = "constants::default_t_cost")]
    pub t_cost: u32,
    #[serde(default = "constants::default_p_cost")]
    pub p_cost: u32,
    #[serde(default = "constants::default_output_len")]
    pub output_len: Option<usize>,
}

serde_default!(PasswordHash);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VOIP {
    #[serde(
        default = "constants::default_keep_voip_room_keep_duration",
        with = "humantime_serde"
    )]
    pub empty_room_keep_duration: Duration,
    #[serde(default = "constants::default_stun_servers")]
    pub stun_servers: Vec<String>,
    /// Whether TURN server is enabled
    #[serde(default)]
    pub turn_enabled: bool,
    /// TURN server URL (e.g., "turn:example.com:3478")
    #[serde(default = "constants::default_turn_server_url")]
    pub turn_server_url: String,
    /// TURN username for authentication
    #[serde(default = "constants::default_turn_username")]
    pub turn_username: String,
    /// TURN password for authentication
    #[serde(default = "constants::default_turn_password")]
    pub turn_password: String,
    /// TTL for TURN credentials in seconds
    #[serde(default = "constants::default_turn_ttl")]
    pub turn_ttl: u64,
}

serde_default!(VOIP);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbArgCfg {
    #[serde(default = "constants::default_fetch_msg_page_size")]
    pub fetch_msg_page_size: u64,
}

serde_default!(DbArgCfg);

/// Raw struct for deserialization with all serde attributes
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawMainCfg {
    pub redis_cfg: PathBuf,
    pub db_cfg: PathBuf,
    pub rabbitmq_cfg: PathBuf,
    pub user_setting: PathBuf,
    pub http_cfg: PathBuf,
    #[serde(default)]
    pub inherit: Option<String>,
    #[serde(default = "constants::default_clear_interval")]
    pub auto_clean_duration: croner::Cron,
    #[serde(
        default = "constants::default_file_save_time",
        with = "humantime_serde"
    )]
    pub files_save_time: Duration,
    #[serde(default = "constants::default_user_files_store_limit")]
    pub user_files_limit: Size,
    #[serde(default = "constants::default_friends_number_limit")]
    pub friends_number_limit: u32,
    #[serde(default = "constants::default_files_storage_path")]
    pub files_storage_path: PathBuf,
    #[serde(default = "constants::default_enable_file_cache")]
    pub enable_file_cache: bool,
    #[serde(default = "constants::default_enable_hierarchical_storage")]
    pub enable_hierarchical_storage: bool,
    #[serde(default = "constants::default_enable_file_deduplication")]
    pub enable_file_deduplication: bool,
    #[serde(default = "constants::default_enable_metrics")]
    pub enable_metrics: bool,
    #[serde(
        default = "constants::default_metrics_snapshot_interval",
        with = "humantime_serde"
    )]
    pub metrics_snapshot_interval: Duration,
    #[serde(default = "constants::default_cache_max_size")]
    pub cache_max_size: Size,
    #[serde(
        default = "constants::default_verification_expire_time",
        with = "humantime_serde"
    )]
    pub verification_expire_time: Duration,
    #[serde(
        default = "constants::default_user_defined_status_expire_time",
        with = "humantime_serde"
    )]
    pub user_defined_status_expire_time: Duration,
    #[serde(
        default = "constants::default_log_clean_duration",
        with = "humantime_serde"
    )]
    pub log_clean_duration: Duration,
    #[serde(default = "constants::default_log_keep", with = "humantime_serde")]
    pub log_keep: Duration,
    #[serde(default = "constants::default_single_instance")]
    pub single_instance: bool,
    #[serde(default = "constants::default_leader_node")]
    pub leader_node: bool,
    #[serde(
        default = "constants::default_room_key_duration",
        with = "humantime_serde"
    )]
    pub room_key_duration: Duration,
    #[serde(default)]
    pub unregister_policy: UnregisterPolicy,
    #[serde(default)]
    pub password_hash: PasswordHash,
    #[serde(default)]
    pub db: DbArgCfg,
    #[serde(default)]
    pub debug: DebugCfg,
    #[serde(default)]
    pub voip: VOIP,
    #[serde(default)]
    pub oauth: OAuthCfg,
    #[serde(default = "constants::default_require_email_verification")]
    pub require_email_verification: bool,
    #[serde(default)]
    pub default_session: Option<SessionID>,

    #[serde(default = "constants::default_lock_account_after_failed_logins")]
    pub lock_account_after_failed_logins: u32,
    #[serde(
        default = "constants::default_lock_account_duration",
        with = "humantime_serde"
    )]
    pub lock_account_duration: Duration,
    #[serde(default)]
    pub initial_admin_ocid: Option<OCID>,
}

impl<'de> Deserialize<'de> for MainCfg {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawMainCfg::deserialize(deserializer)?;

        // Validate configuration
        if raw.friends_number_limit == 0 {
            return Err(D::Error::custom(
                "friends_number_limit must be greater than 0",
            ));
        }
        if raw.files_save_time.is_zero() {
            return Err(D::Error::custom("files_save_time cannot be zero"));
        }
        if raw.cache_max_size.bytes() == 0 {
            return Err(D::Error::custom("cache_max_size cannot be zero"));
        }
        if raw.room_key_duration.is_zero() {
            return Err(D::Error::custom("room_key_duration cannot be zero"));
        }
        if raw.verification_expire_time.is_zero() {
            return Err(D::Error::custom("verification_expire_time cannot be zero"));
        }
        if raw.db.fetch_msg_page_size == 0 {
            return Err(D::Error::custom(
                "fetch_msg_page_size must be greater than 0",
            ));
        }

        Ok(MainCfg {
            inherit: raw.inherit,
            redis_cfg: raw.redis_cfg,
            db_cfg: raw.db_cfg,
            rabbitmq_cfg: raw.rabbitmq_cfg,
            user_setting: raw.user_setting,
            http_cfg: raw.http_cfg,
            auto_clean_duration: raw.auto_clean_duration,
            files_save_time: raw.files_save_time,
            user_files_limit: raw.user_files_limit,
            friends_number_limit: raw.friends_number_limit,
            files_storage_path: raw.files_storage_path,
            enable_file_cache: raw.enable_file_cache,
            enable_hierarchical_storage: raw.enable_hierarchical_storage,
            enable_file_deduplication: raw.enable_file_deduplication,
            enable_metrics: raw.enable_metrics,
            metrics_snapshot_interval: raw.metrics_snapshot_interval,
            cache_max_size: raw.cache_max_size,
            verification_expire_time: raw.verification_expire_time,
            user_defined_status_expire_time: raw.user_defined_status_expire_time,
            log_clean_duration: raw.log_clean_duration,
            log_keep: raw.log_keep,
            single_instance: raw.single_instance,
            leader_node: raw.leader_node,
            room_key_duration: raw.room_key_duration,
            unregister_policy: raw.unregister_policy,
            password_hash: raw.password_hash,
            db: raw.db,
            debug: raw.debug,
            voip: raw.voip,
            oauth: raw.oauth,
            require_email_verification: raw.require_email_verification,
            default_session: raw.default_session,
            lock_account_after_failed_logins: raw.lock_account_after_failed_logins,
            lock_account_duration: raw.lock_account_duration,
            initial_admin_ocid: raw.initial_admin_ocid,
            cmd_args: ParserCfg::default(),
        })
    }
}

impl MainCfg {
    pub fn new(config_path: Vec<impl Into<PathBuf>>) -> anyhow::Result<Self> {
        let mut iter = config_path.into_iter();
        let cfg_path = if let Some(cfg_path) = iter.next() {
            cfg_path.into()
        } else {
            if let Ok(env) = std::env::var(CONFIG_FILE_ENV_VAR) {
                env
            } else {
                bail!("Please specify config file");
            }
            .into()
        };
        let full_basepath = cfg_path
            .parent()
            .context("Config path should be a file, not a root path")?
            .canonicalize()?;
        // read a config file
        let mut cfg: serde_json::Value = setting::read_config_and_deserialize(&cfg_path)?;
        let mut configs_list = vec![cfg_path];
        for i in iter {
            let i = i.into();
            let merge_cfg: serde_json::Value = setting::read_config_and_deserialize(&i)?;
            cfg = merge_json(cfg, merge_cfg);
            configs_list.push(i);
        }
        let mut cfg: MainCfg =
            serde_json::from_value(cfg).context("Failed to deserialize config")?;
        cfg.cmd_args.config = configs_list;
        // convert the path relevant to the config file to a path relevant to the directory
        cfg.convert_to_abs_path(&full_basepath)?;
        // Validate all file paths exist
        cfg.validate_all_paths()?;
        Ok(cfg)
    }

    pub fn unique_instance(&self) -> bool {
        self.leader_node || self.single_instance
    }

    pub fn get_file_path_from_key(&self, key: &str) -> PathBuf {
        self.files_storage_path.join(key)
    }

    /// Validates that all configured file paths exist
    fn validate_all_paths(&self) -> anyhow::Result<()> {
        // Validate sub-config files exist
        if !self.redis_cfg.exists() {
            bail!("redis_cfg does not exist: {}", self.redis_cfg.display());
        }
        if !self.db_cfg.exists() {
            bail!("db_cfg does not exist: {}", self.db_cfg.display());
        }
        if !self.rabbitmq_cfg.exists() {
            bail!(
                "rabbitmq_cfg does not exist: {}",
                self.rabbitmq_cfg.display()
            );
        }
        if !self.user_setting.exists() {
            bail!(
                "user_setting does not exist: {}",
                self.user_setting.display()
            );
        }
        if !self.http_cfg.exists() {
            bail!("http_cfg does not exist: {}", self.http_cfg.display());
        }

        // Validate files_storage_path (create if needed)
        if !self.files_storage_path.exists() {
            std::fs::create_dir_all(&self.files_storage_path)
                .context("Failed to create files_storage_path")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cfg {
    pub main_cfg: MainCfg,
    pub db_cfg: PostgresDbCfg,
    pub redis_cfg: RedisCfg,
    pub rabbitmq_cfg: RabbitMQCfg,
    pub user_setting: UserSetting,
    pub http_cfg: HttpCfg,
}

impl Cfg {
    pub fn new(main_cfg: MainCfg) -> anyhow::Result<Self> {
        let db_cfg = PostgresDbCfg::build_from_path(&main_cfg.db_cfg)?;
        let redis_cfg = RedisCfg::build_from_path(&main_cfg.redis_cfg)?;
        let rabbitmq_cfg = RabbitMQCfg::build_from_path(&main_cfg.rabbitmq_cfg)?;
        let user_setting = UserSetting::build_from_path(&main_cfg.user_setting)?;
        let mut http_cfg = HttpCfg::build_from_path(&main_cfg.http_cfg)?;
        http_cfg.convert_to_abs_path(
            main_cfg
                .http_cfg
                .parent()
                .context("The path of http_cfg is invalid")?,
        )?;
        // Validate http_cfg paths exist
        http_cfg.validate_paths()?;
        Ok(Self {
            main_cfg,
            db_cfg,
            redis_cfg,
            rabbitmq_cfg,
            user_setting,
            http_cfg,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Helper to create a minimal valid config JSON
    fn minimal_valid_config() -> serde_json::Value {
        json!({
            "redis_cfg": "/tmp/redis.toml",
            "db_cfg": "/tmp/db.toml",
            "rabbitmq_cfg": "/tmp/rabbitmq.toml",
            "user_setting": "/tmp/user.toml",
            "http_cfg": "/tmp/http.toml",
            "auto_clean_duration": "0 0 * * * *",
            "files_save_time": "7d",
            "user_files_limit": "1GB",
            "friends_number_limit": 100,
            "files_storage_path": "/tmp/files",
            "cache_max_size": "1GB",
            "verification_expire_time": "1h",
            "user_defined_status_expire_time": "1d",
            "log_clean_duration": "1d",
            "log_keep": "7d",
            "room_key_duration": "1d",
            "db": {
                "fetch_msg_page_size": 50
            }
        })
    }

    #[test]
    fn test_valid_config_deserializes() {
        let config = minimal_valid_config();
        let result: Result<MainCfg, _> = serde_json::from_value(config);
        assert!(
            result.is_ok(),
            "Valid config should deserialize successfully"
        );
        let cfg = result.unwrap();
        assert_eq!(cfg.friends_number_limit, 100);
    }

    #[test]
    fn test_friends_number_limit_zero_fails() {
        let mut config = minimal_valid_config();
        config["friends_number_limit"] = json!(0);
        let result: Result<MainCfg, _> = serde_json::from_value(config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("friends_number_limit must be greater than 0"),
            "Error was: {}",
            err
        );
    }

    #[test]
    fn test_files_save_time_zero_fails() {
        let mut config = minimal_valid_config();
        config["files_save_time"] = json!("0s");
        let result: Result<MainCfg, _> = serde_json::from_value(config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("files_save_time cannot be zero"));
    }

    #[test]
    fn test_cache_max_size_zero_fails() {
        let mut config = minimal_valid_config();
        config["cache_max_size"] = json!("0B");
        let result: Result<MainCfg, _> = serde_json::from_value(config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("cache_max_size cannot be zero"));
    }

    #[test]
    fn test_room_key_duration_zero_fails() {
        let mut config = minimal_valid_config();
        config["room_key_duration"] = json!("0s");
        let result: Result<MainCfg, _> = serde_json::from_value(config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("room_key_duration cannot be zero"));
    }

    #[test]
    fn test_verification_expire_time_zero_fails() {
        let mut config = minimal_valid_config();
        config["verification_expire_time"] = json!("0s");
        let result: Result<MainCfg, _> = serde_json::from_value(config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("verification_expire_time cannot be zero"));
    }

    #[test]
    fn test_fetch_msg_page_size_zero_fails() {
        let mut config = minimal_valid_config();
        config["db"]["fetch_msg_page_size"] = json!(0);
        let result: Result<MainCfg, _> = serde_json::from_value(config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("fetch_msg_page_size must be greater than 0"));
    }

    #[test]
    fn test_multiple_validation_errors_first_is_returned() {
        let mut config = minimal_valid_config();
        config["friends_number_limit"] = json!(0);
        config["files_save_time"] = json!("0s");
        config["db"]["fetch_msg_page_size"] = json!(0);
        let result: Result<MainCfg, _> = serde_json::from_value(config);
        assert!(result.is_err());
        // Should fail on first validation error
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("friends_number_limit must be greater than 0"),
            "Error was {}",
            err
        );
    }
}
