use crate::config::Cfg;
use crate::process::error_msg::SERVER_ERROR;
use crate::server::ServerManageServiceProvider;
use anyhow::Context;
use migration::predefined::PredefinedServerManagementPermission;
use pb::service::server_manage::config::v1::{SetConfigRequest, SetConfigResponse};
use std::path::PathBuf;
use tonic::{Request, Response, Status};
use tracing::{error, info};

#[derive(Debug, thiserror::Error)]
enum SetConfigError {
    #[error("database error:{0:?}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("internal error:{0:?}")]
    InternalError(#[from] anyhow::Error),
    #[error("permission denied")]
    PermissionDenied,
    #[error("invalid JSON: {0:?}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("config validation failed: {0:?}")]
    ValidationFailed(String),
}

/// Fields that require server restart when changed
const RESTART_REQUIRED_FIELDS: &[&str] = &[
    // Database configuration
    "db_cfg",
    "db_cfg.host",
    "db_cfg.port",
    "db_cfg.user",
    "db_cfg.passwd",
    "db_cfg.db",
    // Redis configuration
    "redis_cfg",
    "redis_cfg.host",
    "redis_cfg.port",
    "redis_cfg.user",
    "redis_cfg.passwd",
    // RabbitMQ configuration
    "rabbitmq_cfg",
    "rabbitmq_cfg.host",
    "rabbitmq_cfg.port",
    "rabbitmq_cfg.user",
    "rabbitmq_cfg.passwd",
    "rabbitmq_cfg.vhost",
    // HTTP server configuration
    "http_cfg",
    "http_cfg.ip",
    "http_cfg.port",
    "http_cfg.tls_enable",
    "http_cfg.client_certificate_required",
    // File storage path
    "files_storage_path",
    // Auto clean cron schedule
    "auto_clean_duration",
    // User setting configuration
    "user_setting",
];

/// Check if a config field path requires restart
fn requires_restart(field_path: &str) -> bool {
    RESTART_REQUIRED_FIELDS.iter().any(|restart_field| {
        field_path == *restart_field || field_path.starts_with(&format!("{}.", restart_field))
    })
}

/// Collect all field paths from a JSON value recursively
fn collect_field_paths(value: &serde_json::Value, prefix: &str) -> Vec<String> {
    let mut paths = Vec::new();
    match value {
        serde_json::Value::Object(map) => {
            for (key, val) in map {
                let full_path = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                paths.push(full_path.clone());
                paths.extend(collect_field_paths(val, &full_path));
            }
        }
        serde_json::Value::Array(arr) => {
            for (idx, val) in arr.iter().enumerate() {
                let full_path = format!("{}[{}]", prefix, idx);
                paths.extend(collect_field_paths(val, &full_path));
            }
        }
        _ => {}
    }
    paths
}

/// Find which fields changed between two JSON values
fn find_changed_fields(
    old: &serde_json::Value,
    new: &serde_json::Value,
    prefix: &str,
) -> Vec<String> {
    let mut changed = Vec::new();
    match (old, new) {
        (serde_json::Value::Object(old_map), serde_json::Value::Object(new_map)) => {
            for (key, new_val) in new_map {
                let full_path = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                match old_map.get(key) {
                    Some(old_val) if old_val != new_val => {
                        changed.push(full_path.clone());
                        changed.extend(find_changed_fields(old_val, new_val, &full_path));
                    }
                    None => {
                        // New field added
                        changed.push(full_path.clone());
                        changed.extend(collect_field_paths(new_val, &full_path));
                    }
                    Some(_) => {
                        // No change, but check nested fields
                        changed.extend(find_changed_fields(
                            old_map.get(key).unwrap(),
                            new_val,
                            &full_path,
                        ));
                    }
                }
            }
        }
        (serde_json::Value::Array(old_arr), serde_json::Value::Array(new_arr)) => {
            for (idx, new_val) in new_arr.iter().enumerate() {
                if let Some(old_val) = old_arr.get(idx) {
                    if old_val != new_val {
                        changed.extend(find_changed_fields(old_val, new_val, prefix));
                    }
                }
            }
        }
        _ => {
            // Primitive value changed at this path
            if !prefix.is_empty() && old != new {
                changed.push(prefix.to_string());
            }
        }
    }
    changed
}

async fn set_config_impl(
    server: &ServerManageServiceProvider,
    request: Request<SetConfigRequest>,
) -> Result<SetConfigResponse, SetConfigError> {
    // Get admin user ID from request metadata
    let admin_id =
        crate::process::get_id_from_req(&request).ok_or(SetConfigError::PermissionDenied)?;

    // Check if admin has modify configuration permission
    if !crate::db::manager::manage_permission_existed(
        admin_id,
        PredefinedServerManagementPermission::ModifyConfiguration as i64,
        &server.db.db_pool,
    )
    .await?
    {
        return Err(SetConfigError::PermissionDenied);
    }

    let req = request.into_inner();
    let content = req.content;

    // Parse incoming JSON content
    let new_config_json: serde_json::Value = serde_json::from_str(&content)?;

    // Get current config as JSON
    let current_config_json = serde_json::to_value(&*server.shared_data.cfg())?;

    // Merge configs - use new config for the fields provided
    let merged_json = utils::merge_json(current_config_json.clone(), new_config_json.clone());

    // Validate by deserializing to Cfg
    let merged_cfg: Cfg = serde_json::from_value(merged_json.clone())
        .map_err(|e| SetConfigError::ValidationFailed(e.to_string()))?;

    // Find which fields changed
    let changed_fields = find_changed_fields(&current_config_json, &new_config_json, "");

    // Determine if restart is needed
    let restart_reasons: Vec<String> = changed_fields
        .iter()
        .filter(|path| requires_restart(path))
        .map(|path| format!("Field '{}' requires restart", path))
        .collect();

    let requires_restart = !restart_reasons.is_empty();

    // Write patch file
    let patches_dir = server.shared_data.cfg().main_cfg.patches_directory.clone();
    let patches_dir_path = PathBuf::from(patches_dir);

    // Create patches directory if it doesn't exist
    if !patches_dir_path.exists() {
        tokio::fs::create_dir_all(&patches_dir_path)
            .await
            .context("Failed to create patches directory")?
    }

    // Generate patch filename with timestamp
    let timestamp = chrono::Utc::now().timestamp();

    let patch_filename = format!("config_patch.{}.json", timestamp);
    let patch_path = patches_dir_path.join(&patch_filename);

    // Write the patch file (only contains the changed fields, not the full merged config)
    let patch_content =
        serde_json::to_string_pretty(&new_config_json).context("Failed to serialize patch")?;

    tokio::fs::write(&patch_path, patch_content)
        .await
        .context("Failed to write patch file")?;

    info!("Config patch written to: {}", patch_path.display());

    // Update in-memory config atomically
    // We need to reconstruct the config from merged JSON
    // Since Cfg contains MainCfg which has complex nested structures,
    // we need to update the config through a write lock
    {
        let mut cfg_write = server.shared_data.cfg.write();
        *cfg_write = merged_cfg;
    }

    let message = if requires_restart {
        format!(
            "Configuration updated. Server restart required for: {}",
            restart_reasons.join(", ")
        )
    } else {
        "Configuration updated successfully. Changes applied immediately.".to_string()
    };

    Ok(SetConfigResponse {
        success: true,
        message,
        requires_restart,
        restart_reasons,
    })
}

pub async fn set_config(
    server: &ServerManageServiceProvider,
    request: Request<SetConfigRequest>,
) -> Result<Response<SetConfigResponse>, Status> {
    match set_config_impl(server, request).await {
        Ok(response) => Ok(Response::new(response)),
        Err(e) => match e {
            SetConfigError::PermissionDenied => Err(Status::permission_denied(
                crate::process::error_msg::PERMISSION_DENIED,
            )),
            SetConfigError::ValidationFailed(msg) => {
                error!("Config validation failed: {}", msg);
                Err(Status::invalid_argument(msg))
            }
            SetConfigError::InvalidJson(err) => {
                error!("Invalid JSON in config update: {}", err);
                Err(Status::invalid_argument(format!("Invalid JSON: {}", err)))
            }
            _ => {
                error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
        },
    }
}
