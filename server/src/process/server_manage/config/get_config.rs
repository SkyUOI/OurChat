use crate::process::error_msg::SERVER_ERROR;
use crate::server::ServerManageServiceProvider;
use anyhow::Context;
use migration::predefined::PredefinedServerManagementPermission;
use pb::service::server_manage::config::v1::{GetConfigRequest, GetConfigResponse};
use serde_json::Value;
use tonic::{Request, Response, Status};
use tracing::error;

/// Dotted JSON paths of fields that must never be returned in plaintext by
/// `GetConfig`. Even with the `ViewConfiguration` permission the caller should
/// only see a placeholder, not the live credential (defense in depth against
/// log/leak exfiltration and over-broad roles).
const SENSITIVE_CONFIG_PATHS: &[&[&str]] = &[
    &["db_cfg", "passwd"],
    &["redis_cfg", "passwd"],
    &["rabbitmq_cfg", "passwd"],
    &["main_cfg", "oauth", "github_client_secret"],
    &["main_cfg", "voip", "turn_password"],
    &["main_cfg", "voip", "turn_static_auth_secret"],
];

const MASK: &str = "***";

/// Recursively mask every sensitive path listed above inside `value`.
fn mask_sensitive(value: &mut Value) {
    for path in SENSITIVE_CONFIG_PATHS {
        mask_path(value, path);
    }
}

/// Walk `value` following `path`; whenever the final segment is reached, replace
/// the scalar with the mask placeholder. Missing paths are silently skipped.
fn mask_path(value: &mut Value, path: &[&str]) {
    let Some((first, rest)) = path.split_first() else {
        return;
    };
    let Some(child) = value.get_mut(first) else {
        return;
    };
    if rest.is_empty() {
        // Only mask non-null scalar values so callers can still tell whether a
        // secret is configured at all.
        if !child.is_null() {
            *child = Value::String(MASK.to_string());
        }
        return;
    }
    mask_path(child, rest);
}

#[derive(Debug, thiserror::Error)]
enum GetConfigError {
    #[error("database error:{0:?}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("internal error:{0:?}")]
    InternalError(#[from] anyhow::Error),
    #[error("permission denied")]
    PermissionDenied,
}

async fn get_config_impl(
    server: &ServerManageServiceProvider,
    request: Request<GetConfigRequest>,
) -> Result<GetConfigResponse, GetConfigError> {
    // Get admin user ID from request metadata
    let admin_id =
        crate::process::get_id_from_req(&request).ok_or(GetConfigError::PermissionDenied)?;

    // Check if admin has view configuration permission
    if !crate::db::manager::manage_permission_existed(
        admin_id,
        PredefinedServerManagementPermission::ViewConfiguration as i64,
        &server.db.db_pool,
    )
    .await?
    {
        return Err(GetConfigError::PermissionDenied);
    }
    // Serialize to a JSON tree, mask all secrets, then stringify. This guarantees
    // that no password/secret can leak through the response even though the
    // caller holds the ViewConfiguration permission.
    let mut value = serde_json::to_value(&*server.shared_data.cfg())
        .context("Fatal: cannot serialize config to json")?;
    mask_sensitive(&mut value);
    let content = serde_json::to_string(&value).context("Fatal: cannot serialize masked config")?;
    Ok(GetConfigResponse { content })
}

pub async fn get_config(
    server: &ServerManageServiceProvider,
    request: Request<GetConfigRequest>,
) -> Result<Response<GetConfigResponse>, Status> {
    match get_config_impl(server, request).await {
        Ok(response) => Ok(Response::new(response)),
        Err(e) => match e {
            GetConfigError::PermissionDenied => Err(Status::permission_denied(
                crate::process::error_msg::PERMISSION_DENIED,
            )),
            _ => {
                error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_replaces_secrets() {
        let mut v = serde_json::json!({
            "db_cfg": { "passwd": "s3cret", "user": "postgres" },
            "redis_cfg": { "passwd": "redispw" },
            "rabbitmq_cfg": { "passwd": "mqpw" },
            "main_cfg": {
                "oauth": { "github_client_secret": "gh_secret" },
                "voip": { "turn_password": "turnpw" }
            }
        });
        mask_sensitive(&mut v);
        assert_eq!(v["db_cfg"]["passwd"], MASK);
        assert_eq!(v["db_cfg"]["user"], "postgres");
        assert_eq!(v["redis_cfg"]["passwd"], MASK);
        assert_eq!(v["rabbitmq_cfg"]["passwd"], MASK);
        assert_eq!(v["main_cfg"]["oauth"]["github_client_secret"], MASK);
        assert_eq!(v["main_cfg"]["voip"]["turn_password"], MASK);
    }

    #[test]
    fn mask_skips_missing_and_null() {
        let mut v = serde_json::json!({ "db_cfg": { "passwd": null } });
        mask_sensitive(&mut v);
        assert!(v["db_cfg"]["passwd"].is_null());
    }
}
