use crate::{
    process::error_msg::{self, SERVER_ERROR},
    rabbitmq::{WEBRTC_SIGNAL_EXCHANGE, generate_webrtc_route_key},
    server::RpcServer,
};
use base::constants::ID;
use deadpool_lapin::lapin::BasicProperties;
use deadpool_lapin::lapin::options::BasicPublishOptions;
use pb::service::ourchat::webrtc::signal::v1::{SignalRequest, SignalResponse, SignalType};
use serde::{Deserialize, Serialize};
use tonic::{Request, Response, Status};

#[derive(Serialize, Deserialize)]
struct WebRTCSignalMessage {
    room_id: u64,
    from_user_id: u64,
    to_user_id: u64,
    signal_type: i32,
    sdp: Option<String>,
    ice_candidate: Option<String>,
    sdp_mid: Option<String>,
    sdp_mline_index: Option<u32>,
}

pub async fn signal(
    server: &RpcServer,
    id: ID,
    request: Request<SignalRequest>,
) -> Result<Response<SignalResponse>, Status> {
    match signal_impl(server, id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            SignalErr::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            SignalErr::Status(status) => Err(status),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum SignalErr {
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
}

async fn signal_impl(
    server: &RpcServer,
    from_user_id: ID,
    request: Request<SignalRequest>,
) -> Result<SignalResponse, SignalErr> {
    let req = request.into_inner();

    // Validate signal type has appropriate data
    let signal_type = SignalType::try_from(req.signal_type).unwrap_or(SignalType::Unspecified);

    match signal_type {
        SignalType::Offer | SignalType::Answer => {
            if req.sdp.is_empty() {
                return Err(SignalErr::Status(Status::invalid_argument(
                    "sdp is required for offer/answer signals",
                )));
            }
        }
        SignalType::IceCandidate => {
            if req.ice_candidate.is_empty() {
                return Err(SignalErr::Status(Status::invalid_argument(
                    "ice_candidate is required for ICE candidate signals",
                )));
            }
        }
        SignalType::Unspecified => {
            return Err(SignalErr::Status(Status::invalid_argument(
                error_msg::webrtc::UNSPECIFIED,
            )));
        }
    }

    // Create signal message
    let signal_msg = WebRTCSignalMessage {
        room_id: req.room_id,
        from_user_id: *from_user_id,
        to_user_id: req.target_user_id,
        signal_type: req.signal_type,
        sdp: if req.sdp.is_empty() {
            None
        } else {
            Some(req.sdp)
        },
        ice_candidate: if req.ice_candidate.is_empty() {
            None
        } else {
            Some(req.ice_candidate)
        },
        sdp_mid: if req.sdp_mid.is_empty() {
            None
        } else {
            Some(req.sdp_mid)
        },
        sdp_mline_index: if req.sdp_mline_index == 0 {
            None
        } else {
            Some(req.sdp_mline_index)
        },
    };

    let signal_bytes = serde_json::to_vec(&signal_msg)
        .map_err(|e| anyhow::anyhow!("json serialization error: {}", e))?;

    // Publish to RabbitMQ for the target user
    let rmq_conn = server
        .rabbitmq
        .get()
        .await
        .map_err(|e| anyhow::anyhow!("rabbitmq pool error: {:?}", e))?;
    let channel = rmq_conn
        .create_channel()
        .await
        .map_err(|e| anyhow::anyhow!("rabbitmq channel error: {:?}", e))?;
    let routing_key = generate_webrtc_route_key(req.target_user_id.into());

    channel
        .basic_publish(
            WEBRTC_SIGNAL_EXCHANGE,
            &routing_key,
            BasicPublishOptions::default(),
            &signal_bytes,
            BasicProperties::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("rabbitmq publish error: {:?}", e))?;

    Ok(SignalResponse { success: true })
}
