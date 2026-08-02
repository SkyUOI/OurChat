use crate::server::BasicServiceProvider;
use base::constants::{ID, JWT_HEADER};
use base64::Engine;
use hmac::{Hmac, KeyInit, Mac};
use pb::service::basic::voip::v1::{GetVoipConfigRequest, GetVoipConfigResponse};
use sha1::Sha1;
use tonic::{Request, Response, Status};

/// coturn REST-API HMAC key type.
type HmacSha1 = Hmac<Sha1>;

/// Verify the caller's JWT (BasicService has no auth interceptor, so this is
/// done manually) and return the authenticated user's ID.
fn authenticate(request: &Request<GetVoipConfigRequest>) -> Result<ID, Status> {
    let token = request
        .metadata()
        .get(JWT_HEADER)
        .ok_or_else(|| Status::unauthenticated(crate::process::error_msg::token::MISSING))?;
    let token = token
        .to_str()
        .map_err(|_| Status::invalid_argument(crate::process::error_msg::token::INVALID))?;
    let jwt = crate::process::check_token(token).map_err(|e| {
        tracing::info!(error = %e, "GetVoipConfig: authentication failed");
        match e {
            crate::process::ErrAuth::Expire => {
                Status::unauthenticated(crate::process::error_msg::token::EXPIRED)
            }
            _ => Status::unauthenticated(crate::process::error_msg::token::INVALID),
        }
    })?;
    Ok(jwt.id)
}

/// Generate a coturn-compatible time-limited credential pair.
///
/// Format (matches coturn `--use-auth-secret --static-auth-secret <secret>`):
/// * `username = "{expiry_unix}:{userid}"`
/// * `password = base64( HMAC-SHA1(secret, username) )`
///
/// The credential is valid until `expiry_unix` (unix seconds) and is bound to
/// the authenticated user, so it cannot be shared anonymously.
fn generate_ephemeral_credential(
    secret: &str,
    user_id: ID,
    ttl_seconds: u64,
    now_unix: i64,
) -> Result<(String, String), Status> {
    let expiry = now_unix.saturating_add(ttl_seconds.max(1) as i64);
    let username = format!("{expiry}:{}", *user_id);
    let mut mac = HmacSha1::new_from_slice(secret.as_bytes()).map_err(|e| {
        tracing::error!("invalid TURN static auth secret length: {e}");
        Status::internal(crate::process::error_msg::SERVER_ERROR)
    })?;
    mac.update(username.as_bytes());
    let password = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
    Ok((username, password))
}

pub async fn get_voip_config(
    server: &BasicServiceProvider,
    request: Request<GetVoipConfigRequest>,
) -> Result<Response<GetVoipConfigResponse>, Status> {
    // Require authentication: TURN credentials must never be handed out to
    // anonymous clients (they can be abused as open traffic relays).
    let user_id = authenticate(&request)?;

    let config = server.shared_data.cfg();
    let voip = &config.main_cfg.voip;
    let ttl = voip.turn_ttl;

    let (username, password) = match &voip.turn_static_auth_secret {
        Some(secret) if !secret.is_empty() => {
            // Issue a per-user, time-limited credential.
            let now = chrono::Utc::now().timestamp();
            generate_ephemeral_credential(secret, user_id, ttl, now)?
        }
        _ => {
            // No static-auth-secret configured: fall back to the shared static
            // credential, but only for the authenticated caller. A warning is
            // logged because this leaks a long-lived shared secret to users.
            tracing::warn!(
                "GetVoipConfig: serving static TURN credential to user {} because \
                 turn_static_auth_secret is not configured; set it to enable \
                 per-user time-limited credentials",
                user_id
            );
            (voip.turn_username.clone(), voip.turn_password.clone())
        }
    };

    Ok(Response::new(GetVoipConfigResponse {
        stun_servers: voip.stun_servers.clone(),
        turn_enabled: voip.turn_enabled,
        turn_server_url: voip.turn_server_url.clone(),
        turn_username: username,
        turn_password: password,
        turn_ttl: ttl,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ephemeral_credential_is_user_scoped_and_deterministic() {
        let now = 1_700_000_000;
        let (u, p) = generate_ephemeral_credential("topsecret", ID(42), 3600, now).unwrap();
        // username encodes expiry and user id
        assert_eq!(u, format!("{}:42", now + 3600));
        // same inputs -> same password
        let (_, p2) = generate_ephemeral_credential("topsecret", ID(42), 3600, now).unwrap();
        assert_eq!(p, p2);
        // different user -> different password
        let (_, p3) = generate_ephemeral_credential("topsecret", ID(43), 3600, now).unwrap();
        assert_ne!(p, p3);
    }
}
