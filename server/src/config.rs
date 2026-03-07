mod http;

use std::{path::PathBuf, time::Duration};

use anyhow::{Context, bail};
use base::constants::OCID;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, de::DeserializeOwned};
use size::Size;
use utils::{merge_json, resolve_relative_path, serde_default};

use crate::{ParserCfg, config::http::HttpCfg};
use base::{
    constants::{self, CONFIG_FILE_ENV_VAR, SessionID},
    database::{postgres::PostgresDbCfg, redis_cfg::RedisCfg},
    rabbitmq::RabbitMQCfg,
    setting::{self, PathConvert, Setting, UserSetting, debug::DebugCfg},
};

/// A configuration source that can be either a path to a config file or an inline configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub enum ConfigSource<T> {
    Path(PathBuf),
    Inline(T),
}

impl<T> ConfigSource<T> {
    /// Load configuration from either a path or inline value.
    pub fn load(&self) -> anyhow::Result<T>
    where
        T: Setting + Clone + DeserializeOwned,
    {
        match self {
            ConfigSource::Path(path) => T::build_from_path(path),
            ConfigSource::Inline(config) => Ok(config.clone()),
        }
    }
}

impl<T> PathConvert for ConfigSource<T>
where
    T: PathConvert,
{
    fn convert_to_abs_path(&mut self, full_basepath: &std::path::Path) -> anyhow::Result<()> {
        match self {
            ConfigSource::Path(path) => {
                *path = resolve_relative_path(full_basepath, path)?;
            }
            ConfigSource::Inline(config) => {
                config.convert_to_abs_path(full_basepath)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Clone, derive::PathConvert)]
pub struct MainCfg {
    pub inherit: Option<String>,
    #[path_convert]
    pub redis_cfg: ConfigSource<RedisCfg>,
    #[path_convert]
    pub db_cfg: ConfigSource<PostgresDbCfg>,
    #[path_convert]
    pub rabbitmq_cfg: ConfigSource<RabbitMQCfg>,
    #[path_convert]
    pub user_setting: ConfigSource<UserSetting>,
    #[path_convert]
    pub http_cfg: ConfigSource<HttpCfg>,
    #[serde(default = "constants::default_clear_interval")]
    pub auto_clean_duration: croner::Cron,
    #[serde(with = "humantime_serde")]
    pub files_save_time: Duration,
    pub user_files_limit: Size,
    pub friends_number_limit: u32,
    pub files_storage_path: PathBuf,
    pub enable_file_cache: bool,
    pub enable_hierarchical_storage: bool,
    pub enable_file_deduplication: bool,
    pub enable_metrics: bool,
    #[serde(with = "humantime_serde")]
    pub metrics_snapshot_interval: Duration,
    pub cache_max_size: Size,
    #[serde(with = "humantime_serde")]
    pub verification_expire_time: Duration,
    #[serde(with = "humantime_serde")]
    pub user_defined_status_expire_time: Duration,
    #[serde(with = "humantime_serde")]
    pub log_clean_duration: Duration,
    #[serde(with = "humantime_serde")]
    pub log_keep: Duration,
    pub single_instance: bool,
    pub leader_node: bool,
    #[serde(with = "humantime_serde")]
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
    #[serde(with = "humantime_serde")]
    pub lock_account_duration: Duration,
    pub initial_admin_ocid: Option<OCID>,
    #[serde(default = "constants::default_patches_directory")]
    pub patches_directory: String,

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
    pub redis_cfg: ConfigSource<RedisCfg>,
    pub db_cfg: ConfigSource<PostgresDbCfg>,
    pub rabbitmq_cfg: ConfigSource<RabbitMQCfg>,
    pub user_setting: ConfigSource<UserSetting>,
    pub http_cfg: ConfigSource<HttpCfg>,
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
    #[serde(default = "constants::default_patches_directory")]
    pub patches_directory: String,
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
            patches_directory: raw.patches_directory,
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

        // Load and apply patches from patches_directory
        cfg.load_and_apply_patches()?;

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

    /// Load and apply patches from the patches_directory.
    /// Patches are JSON files with names like "config_patch.1234567890.json"
    /// They are applied in timestamp order (oldest to newest).
    fn load_and_apply_patches(&mut self) -> anyhow::Result<()> {
        let patches_dir = PathBuf::from(&self.patches_directory);

        // If patches directory doesn't exist, no patches to apply
        if !patches_dir.exists() {
            return Ok(());
        }

        // Create patches directory if it doesn't exist (for future patches)
        if !patches_dir.is_dir() {
            bail!(
                "patches_directory '{}' exists but is not a directory",
                self.patches_directory
            );
        }

        // Read all patch files matching the pattern config_patch.*.json
        let mut patch_files: Vec<(PathBuf, u64)> = Vec::new();
        let entries = std::fs::read_dir(&patches_dir).with_context(|| {
            format!(
                "Failed to read patches directory: {}",
                patches_dir.display()
            )
        })?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // Only process regular files
            if !path.is_file() {
                continue;
            }

            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Check if filename matches pattern "config_patch.{timestamp}.json"
            if let Some(timestamp_str) = file_name
                .strip_prefix("config_patch.")
                .and_then(|s| s.strip_suffix(".json"))
            {
                if let Ok(timestamp) = timestamp_str.parse::<u64>() {
                    patch_files.push((path, timestamp));
                }
            }
        }

        // Sort by timestamp (oldest first)
        patch_files.sort_by_key(|(_, ts)| *ts);

        // Apply each patch in order
        for (patch_path, timestamp) in patch_files {
            tracing::info!("Applying config patch from: {}", patch_path.display());

            let patch_content = std::fs::read_to_string(&patch_path)
                .with_context(|| format!("Failed to read patch file: {}", patch_path.display()))?;

            let patch_json: serde_json::Value =
                serde_json::from_str(&patch_content).with_context(|| {
                    format!(
                        "Failed to parse patch file as JSON: {}",
                        patch_path.display()
                    )
                })?;

            // Convert current config to JSON
            let current_json =
                serde_json::to_value(&*self).context("Failed to serialize current config")?;

            // Merge patch with current config
            let merged_json = merge_json(current_json, patch_json);

            // Deserialize merged config back to MainCfg
            *self = serde_json::from_value(merged_json)
                .context("Failed to deserialize merged config")?;

            tracing::info!(
                "Successfully applied config patch with timestamp {}",
                timestamp
            );
        }

        Ok(())
    }

    /// Validates that all configured file paths exist
    fn validate_all_paths(&self) -> anyhow::Result<()> {
        // Validate sub-config files exist (only for Path variants)
        if let ConfigSource::Path(path) = &self.redis_cfg
            && !path.exists()
        {
            bail!("redis_cfg does not exist: {}", path.display());
        }
        if let ConfigSource::Path(path) = &self.db_cfg
            && !path.exists()
        {
            bail!("db_cfg does not exist: {}", path.display());
        }
        if let ConfigSource::Path(path) = &self.rabbitmq_cfg
            && !path.exists()
        {
            bail!("rabbitmq_cfg does not exist: {}", path.display());
        }
        if let ConfigSource::Path(path) = &self.user_setting
            && !path.exists()
        {
            bail!("user_setting does not exist: {}", path.display());
        }
        if let ConfigSource::Path(path) = &self.http_cfg
            && !path.exists()
        {
            bail!("http_cfg does not exist: {}", path.display());
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
        let db_cfg = main_cfg.db_cfg.load()?;
        let redis_cfg = main_cfg.redis_cfg.load()?;
        let rabbitmq_cfg = main_cfg.rabbitmq_cfg.load()?;
        let user_setting = main_cfg.user_setting.load()?;
        let mut http_cfg = main_cfg.http_cfg.load()?;
        if let ConfigSource::Path(ref path) = main_cfg.http_cfg {
            http_cfg
                .convert_to_abs_path(path.parent().context("The path of http_cfg is invalid")?)?;
        }
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

    #[test]
    fn test_inline_redis_config() {
        let mut config = minimal_valid_config();
        // Replace path string with inline configuration
        config["redis_cfg"] = json!({
            "host": "localhost",
            "port": 6379,
            "passwd": "secret",
            "user": "default"
        });
        let result: Result<MainCfg, _> = serde_json::from_value(config);
        assert!(result.is_ok(), "Inline redis config should deserialize");
        let cfg = result.unwrap();
        // Verify it's the Inline variant
        match &cfg.redis_cfg {
            ConfigSource::Inline(_) => {} // success
            ConfigSource::Path(_) => panic!("Expected inline config, got path"),
        }
    }

    #[test]
    fn test_inline_db_config() {
        let mut config = minimal_valid_config();
        config["db_cfg"] = json!({
            "host": "localhost",
            "port": 5432,
            "db": "ourchat",
            "user": "postgres",
            "passwd": "password"
        });
        let result: Result<MainCfg, _> = serde_json::from_value(config);
        assert!(result.is_ok(), "Inline db config should deserialize");
        let cfg = result.unwrap();
        match &cfg.db_cfg {
            ConfigSource::Inline(_) => {}
            ConfigSource::Path(_) => panic!("Expected inline config, got path"),
        }
    }

    #[test]
    fn test_inline_rabbitmq_config() {
        let mut config = minimal_valid_config();
        config["rabbitmq_cfg"] = json!({
            "host": "localhost",
            "port": 5672,
            "user": "guest",
            "passwd": "guest",
            "vhost": "/"
        });
        let result: Result<MainCfg, _> = serde_json::from_value(config);
        assert!(result.is_ok(), "Inline rabbitmq config should deserialize");
        let cfg = result.unwrap();
        match &cfg.rabbitmq_cfg {
            ConfigSource::Inline(_) => {}
            ConfigSource::Path(_) => panic!("Expected inline config, got path"),
        }
    }

    #[test]
    fn test_inline_user_setting_config() {
        let mut config = minimal_valid_config();
        config["user_setting"] = json!({
            "contacts": [],
            "support_page": "http://example.com",
            "password_strength_limit": 1,
            "verify_email_expiry": "5min",
            "add_friend_request_expiry": "1h"
        });
        let result: Result<MainCfg, _> = serde_json::from_value(config);
        if let Err(ref e) = result {
            println!("Deserialization error: {}", e);
        }
        assert!(
            result.is_ok(),
            "Inline user_setting config should deserialize"
        );
        let cfg = result.unwrap();
        match &cfg.user_setting {
            ConfigSource::Inline(_) => {}
            ConfigSource::Path(_) => panic!("Expected inline config, got path"),
        }
    }

    #[test]
    fn test_inline_http_config() {
        let mut config = minimal_valid_config();
        config["http_cfg"] = json!({
            "ip": "127.0.0.1",
            "port": 8080,
            "logo_path": "/tmp/logo.png",
            "default_avatar_path": "/tmp/avatar.png",
            "log_clean_duration": "1d",
            "lop_keep": "7d"
        });
        let result: Result<MainCfg, _> = serde_json::from_value(config);
        assert!(result.is_ok(), "Inline http config should deserialize");
        let cfg = result.unwrap();
        match &cfg.http_cfg {
            ConfigSource::Inline(_) => {}
            ConfigSource::Path(_) => panic!("Expected inline config, got path"),
        }
    }

    #[test]
    fn test_mixed_config() {
        let mut config = minimal_valid_config();
        // Mix inline and path configurations
        config["redis_cfg"] = json!({
            "host": "localhost",
            "port": 6379,
            "passwd": "secret",
            "user": "default"
        });
        config["db_cfg"] = json!("/tmp/db.toml");
        config["rabbitmq_cfg"] = json!({
            "host": "localhost",
            "port": 5672,
            "user": "guest",
            "passwd": "guest",
            "vhost": "/"
        });
        let result: Result<MainCfg, _> = serde_json::from_value(config);
        assert!(result.is_ok(), "Mixed config should deserialize");
        let cfg = result.unwrap();
        match &cfg.redis_cfg {
            ConfigSource::Inline(_) => {}
            ConfigSource::Path(_) => panic!("Expected inline config for redis, got path"),
        }
        match &cfg.db_cfg {
            ConfigSource::Path(_) => {}
            ConfigSource::Inline(_) => panic!("Expected path config for db, got inline"),
        }
        match &cfg.rabbitmq_cfg {
            ConfigSource::Inline(_) => {}
            ConfigSource::Path(_) => panic!("Expected inline config for rabbitmq, got path"),
        }
    }

    #[test]
    fn test_patches_directory_default_value() {
        let config = minimal_valid_config();
        let result: Result<MainCfg, _> = serde_json::from_value(config);
        assert!(result.is_ok());
        let cfg = result.unwrap();
        assert_eq!(cfg.patches_directory, "./patches");
    }

    #[test]
    fn test_patches_directory_custom_value() {
        let mut config = minimal_valid_config();
        config["patches_directory"] = json!("/custom/patches/path");
        let result: Result<MainCfg, _> = serde_json::from_value(config);
        assert!(result.is_ok());
        let cfg = result.unwrap();
        assert_eq!(cfg.patches_directory, "/custom/patches/path");
    }

    #[test]
    fn test_load_and_apply_patches_no_patches_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_json = minimal_valid_config();
        let mut cfg: MainCfg = serde_json::from_value(config_json).unwrap();

        // Set patches directory to a non-existent path
        cfg.patches_directory = temp_dir
            .path()
            .join("non_existent")
            .to_str()
            .unwrap()
            .to_string();

        // Should not fail when patches directory doesn't exist
        let result = cfg.load_and_apply_patches();
        assert!(
            result.is_ok(),
            "Should succeed when patches directory doesn't exist"
        );
    }

    #[test]
    fn test_load_and_apply_patches_empty_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_json = minimal_valid_config();
        let mut cfg: MainCfg = serde_json::from_value(config_json).unwrap();

        // Set patches directory to existing empty directory
        cfg.patches_directory = temp_dir.path().to_str().unwrap().to_string();

        // Should not fail with empty patches directory
        let result = cfg.load_and_apply_patches();
        assert!(
            result.is_ok(),
            "Should succeed with empty patches directory"
        );
    }

    #[test]
    fn test_load_and_apply_patches_single_patch() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_json = minimal_valid_config();
        let mut cfg: MainCfg = serde_json::from_value(config_json).unwrap();

        // Set patches directory
        let patches_dir = temp_dir.path().join("patches");
        std::fs::create_dir_all(&patches_dir).unwrap();
        cfg.patches_directory = patches_dir.to_str().unwrap().to_string();

        // Create a patch file that changes friends_number_limit
        let patch_path = patches_dir.join("config_patch.1234567890.json");
        let patch = json!({
            "friends_number_limit": 250
        });
        std::fs::write(&patch_path, serde_json::to_string_pretty(&patch).unwrap()).unwrap();

        // Load and apply patches
        let result = cfg.load_and_apply_patches();
        if let Err(e) = &result {
            eprintln!("Error loading patches: {}", e);
            eprintln!("Error chain: {:?}", e.chain().collect::<Vec<_>>());
        }
        assert!(
            result.is_ok(),
            "Should successfully apply patch: {:?}",
            result
        );

        // Verify the patch was applied
        assert_eq!(
            cfg.friends_number_limit, 250,
            "friends_number_limit should be updated by patch"
        );
    }

    #[test]
    fn test_load_and_apply_patches_multiple_patches_ordered() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_json = minimal_valid_config();
        let mut cfg: MainCfg = serde_json::from_value(config_json).unwrap();

        // Set patches directory
        let patches_dir = temp_dir.path().join("patches");
        std::fs::create_dir_all(&patches_dir).unwrap();
        cfg.patches_directory = patches_dir.to_str().unwrap().to_string();

        // Create multiple patch files with different timestamps
        // Note: Files are created out of timestamp order to test sorting
        let patch2_path = patches_dir.join("config_patch.1234567892.json");
        let patch2 = json!({
            "friends_number_limit": 300
        });
        std::fs::write(&patch2_path, serde_json::to_string_pretty(&patch2).unwrap()).unwrap();

        let patch1_path = patches_dir.join("config_patch.1234567891.json");
        let patch1 = json!({
            "friends_number_limit": 200
        });
        std::fs::write(&patch1_path, serde_json::to_string_pretty(&patch1).unwrap()).unwrap();

        let patch3_path = patches_dir.join("config_patch.1234567893.json");
        let patch3 = json!({
            "friends_number_limit": 400
        });
        std::fs::write(&patch3_path, serde_json::to_string_pretty(&patch3).unwrap()).unwrap();

        // Load and apply patches
        let result = cfg.load_and_apply_patches();
        assert!(result.is_ok(), "Should successfully apply multiple patches");

        // Verify patches were applied in order (oldest to newest)
        // The final value should be from patch3 (newest timestamp)
        assert_eq!(
            cfg.friends_number_limit, 400,
            "Should apply newest patch last"
        );
    }

    #[test]
    fn test_load_and_apply_patches_ignores_non_matching_files() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_json = minimal_valid_config();
        let mut cfg: MainCfg = serde_json::from_value(config_json).unwrap();

        // Set patches directory
        let patches_dir = temp_dir.path().join("patches");
        std::fs::create_dir_all(&patches_dir).unwrap();
        cfg.patches_directory = patches_dir.to_str().unwrap().to_string();

        // Create files that don't match the pattern
        std::fs::write(patches_dir.join("other_file.txt"), "not a patch").unwrap();
        std::fs::write(patches_dir.join("config.json"), "{\"key\": \"value\"}").unwrap();
        std::fs::write(patches_dir.join(".hidden_file"), "hidden").unwrap();

        // Create a subdirectory
        std::fs::create_dir(patches_dir.join("subdir")).unwrap();

        // Create a valid patch file
        let patch_path = patches_dir.join("config_patch.1234567890.json");
        let patch = json!({
            "friends_number_limit": 150
        });
        std::fs::write(&patch_path, serde_json::to_string_pretty(&patch).unwrap()).unwrap();

        // Load and apply patches
        let result = cfg.load_and_apply_patches();
        assert!(result.is_ok(), "Should ignore non-matching files");

        // Verify only the valid patch was applied
        assert_eq!(cfg.friends_number_limit, 150);
    }

    #[test]
    fn test_load_and_apply_patches_invalid_json_fails() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_json = minimal_valid_config();
        let mut cfg: MainCfg = serde_json::from_value(config_json).unwrap();

        // Set patches directory
        let patches_dir = temp_dir.path().join("patches");
        std::fs::create_dir_all(&patches_dir).unwrap();
        cfg.patches_directory = patches_dir.to_str().unwrap().to_string();

        // Create a patch file with invalid JSON
        let patch_path = patches_dir.join("config_patch.1234567890.json");
        std::fs::write(&patch_path, "{ invalid json }").unwrap();

        // Load and apply patches should fail
        let result = cfg.load_and_apply_patches();
        assert!(result.is_err(), "Should fail on invalid JSON patch");

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Failed to parse patch file as JSON") || err_msg.contains("expected"),
            "Error should mention JSON parsing failure, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_load_and_apply_patches_merges_nested_values() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_json = minimal_valid_config();
        let mut cfg: MainCfg = serde_json::from_value(config_json).unwrap();

        // Set patches directory
        let patches_dir = temp_dir.path().join("patches");
        std::fs::create_dir_all(&patches_dir).unwrap();
        cfg.patches_directory = patches_dir.to_str().unwrap().to_string();

        // Create a patch that changes nested db config
        let original_page_size = cfg.db.fetch_msg_page_size;
        let patch_path = patches_dir.join("config_patch.1234567890.json");
        let patch = json!({
            "db": {
                "fetch_msg_page_size": 100
            }
        });
        std::fs::write(&patch_path, serde_json::to_string_pretty(&patch).unwrap()).unwrap();

        // Load and apply patches
        let result = cfg.load_and_apply_patches();
        assert!(result.is_ok(), "Should successfully apply nested patch");

        // Verify the nested value was updated
        assert_eq!(
            cfg.db.fetch_msg_page_size, 100,
            "Nested db config should be updated"
        );
        assert_ne!(
            cfg.db.fetch_msg_page_size, original_page_size,
            "Value should have changed"
        );
    }

    #[test]
    fn test_load_and_apply_patches_directory_not_dir_fails() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_json = minimal_valid_config();
        let mut cfg: MainCfg = serde_json::from_value(config_json).unwrap();

        // Create a file instead of a directory
        let file_path = temp_dir.path().join("not_a_directory");
        std::fs::write(&file_path, "I am a file, not a directory").unwrap();

        // Set patches directory to point to a file
        cfg.patches_directory = file_path.to_str().unwrap().to_string();

        // Load and apply patches should fail
        let result = cfg.load_and_apply_patches();
        assert!(
            result.is_err(),
            "Should fail when patches_directory is not a directory"
        );

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not a directory"),
            "Error should mention not a directory, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_serialize_and_deserialize_main_cfg() {
        // Test that MainCfg can be serialized and deserialized correctly
        let config_json = minimal_valid_config();
        let cfg: MainCfg = serde_json::from_value(config_json.clone()).unwrap();

        // Serialize to JSON
        let serialized = serde_json::to_value(&cfg).unwrap();

        // The serialized JSON should have the same friends_number_limit
        assert_eq!(serialized["friends_number_limit"], 100);

        // Should be able to deserialize back
        let deserialized: MainCfg = serde_json::from_value(serialized).unwrap();
        assert_eq!(deserialized.friends_number_limit, 100);
    }

    #[test]
    fn test_merge_json_preserves_all_fields() {
        // Test that merge_json preserves all fields from the original
        use utils::merge_json;

        let config_json = minimal_valid_config();
        let cfg: MainCfg = serde_json::from_value(config_json).unwrap();
        let original_json = serde_json::to_value(&cfg).unwrap();

        // Create a patch that only changes friends_number_limit
        let patch = json!({
            "friends_number_limit": 250
        });

        // Merge
        let merged_json = merge_json(original_json, patch);

        // Verify the merged JSON has all required fields
        assert!(
            merged_json.get("redis_cfg").is_some(),
            "redis_cfg should exist after merge"
        );
        assert!(
            merged_json.get("db_cfg").is_some(),
            "db_cfg should exist after merge"
        );
        assert!(
            merged_json.get("friends_number_limit").is_some(),
            "friends_number_limit should exist after merge"
        );

        // Verify the value was updated
        assert_eq!(merged_json["friends_number_limit"], 250);

        // Should be able to deserialize the merged config
        let merged_cfg: MainCfg = serde_json::from_value(merged_json)
            .expect("Should be able to deserialize merged config");
        assert_eq!(merged_cfg.friends_number_limit, 250);
    }
}
