use crate::server::BasicServiceProvider;
use pb::service::basic::voip::v1::GetVoipConfigResponse;
use tonic::{Response, Status};

pub async fn get_voip_config(
    server: &BasicServiceProvider,
) -> Result<Response<GetVoipConfigResponse>, Status> {
    let config = server.shared_data.cfg();
    let voip = &config.main_cfg.voip;

    Ok(Response::new(GetVoipConfigResponse {
        stun_servers: voip.stun_servers.clone(),
        turn_enabled: voip.turn_enabled,
        turn_server_url: voip.turn_server_url.clone(),
        turn_username: voip.turn_username.clone(),
        turn_password: voip.turn_password.clone(),
        turn_ttl: voip.turn_ttl,
    }))
}
