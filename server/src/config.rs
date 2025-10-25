use std::{path::PathBuf, time::Duration};

use anyhow::bail;
use serde::{Deserialize, Serialize};
use size::Size;
use utils::merge_json;

use crate::{ParserCfg, httpserver::HttpCfg};
use base::{
    consts::{self, CONFIG_FILE_ENV_VAR},
    database::{postgres::PostgresDbCfg, redis::RedisCfg},
    rabbitmq::RabbitMQCfg,
    setting::{self, PathConvert, Setting, UserSetting, debug::DebugCfg},
};

#[derive(Debug, Serialize, Deserialize, Clone, derive::PathConvert)]
pub struct MainCfg {
    pub redis_cfg: PathBuf,
    pub db_cfg: PathBuf,
    pub rabbitmq_cfg: PathBuf,
    pub user_setting: PathBuf,
    pub http_cfg: PathBuf,
    #[serde(default = "consts::default_clear_interval")]
    pub auto_clean_duration: croner::Cron,
    #[serde(default = "consts::default_file_save_time", with = "humantime_serde")]
    pub file_save_time: Duration,
    #[serde(default = "consts::default_user_files_store_limit")]
    pub user_files_limit: Size,
    #[serde(default = "consts::default_friends_number_limit")]
    pub friends_number_limit: u32,
    #[serde(default = "consts::default_files_storage_path")]
    pub files_storage_path: PathBuf,
    #[serde(
        default = "consts::default_verification_expire_time",
        with = "humantime_serde"
    )]
    pub verification_expire_time: Duration,
    #[serde(
        default = "consts::default_user_defined_status_expire_time",
        with = "humantime_serde"
    )]
    pub user_defined_status_expire_time: Duration,
    #[serde(
        default = "consts::default_log_clean_duration",
        with = "humantime_serde"
    )]
    pub log_clean_duration: Duration,
    #[serde(default = "consts::default_log_keep", with = "humantime_serde")]
    pub lop_keep: Duration,
    #[serde(default = "consts::default_single_instance")]
    pub single_instance: bool,
    #[serde(default = "consts::default_leader_node")]
    pub leader_node: bool,
    #[serde(
        default = "consts::default_room_key_duration",
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
pub struct PasswordHash {
    #[serde(default = "consts::default_m_cost")]
    pub m_cost: u32,
    #[serde(default = "consts::default_t_cost")]
    pub t_cost: u32,
    #[serde(default = "consts::default_p_cost")]
    pub p_cost: u32,
    #[serde(default = "consts::default_output_len")]
    pub output_len: Option<usize>,
}

impl Default for PasswordHash {
    fn default() -> Self {
        let empty = serde_json::json!({});
        serde_json::from_value(empty).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VOIP {
    #[serde(
        default = "consts::default_keep_voip_room_keep_duration",
        with = "humantime_serde"
    )]
    pub empty_room_keep_duration: Duration,
}

impl Default for VOIP {
    fn default() -> Self {
        let empty = serde_json::json!({});
        serde_json::from_value(empty).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbArgCfg {
    #[serde(default = "consts::default_fetch_msg_page_size")]
    pub fetch_msg_page_size: u64,
}

impl Default for DbArgCfg {
    fn default() -> Self {
        let empty = serde_json::json!({});
        serde_json::from_value(empty).unwrap()
    }
}

impl MainCfg {
    pub fn new(config_path: Vec<impl Into<PathBuf>>) -> anyhow::Result<Self> {
        let len = config_path.len();
        let mut iter = config_path.into_iter();
        let cfg_path = if len == 0 {
            if let Ok(env) = std::env::var(CONFIG_FILE_ENV_VAR) {
                env
            } else {
                tracing::error!("Please specify config file");
                bail!("Please specify config file");
            }
            .into()
        } else {
            iter.next().unwrap().into()
        };
        // read a config file
        let mut cfg: serde_json::Value = setting::read_config_and_deserialize(&cfg_path)?;
        let mut configs_list = vec![cfg_path];
        for i in iter {
            let i = i.into();
            let merge_cfg: serde_json::Value = setting::read_config_and_deserialize(&i)?;
            cfg = merge_json(cfg, merge_cfg);
            configs_list.push(i);
        }
        let mut cfg: MainCfg = serde_json::from_value(cfg).expect("Failed to deserialize config");
        cfg.cmd_args.config = configs_list;
        // convert the path relevant to the config file to a path relevant to the directory
        let full_basepath = cfg
            .cmd_args
            .config
            .first()
            .unwrap()
            .parent()
            .unwrap()
            .canonicalize()?;
        cfg.convert_to_abs_path(&full_basepath)?;
        Ok(cfg)
    }

    pub fn unique_instance(&self) -> bool {
        self.leader_node || self.single_instance
    }

    pub fn get_file_path_from_key(&self, key: &str) -> PathBuf {
        self.files_storage_path.join(key)
    }
}

#[derive(Debug, Clone)]
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
        http_cfg.convert_to_abs_path(main_cfg.http_cfg.parent().unwrap())?;
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
