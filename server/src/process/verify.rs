use std::sync::Arc;

use tokio::{
    select,
    sync::{Notify, mpsc},
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response};

use crate::{
    component::EmailSender,
    consts::VERIFY_EMAIL_EXPIRE,
    server::{
        AuthServiceProvider, VerifyStream,
        httpserver::verify::{VerifyRecord, verify_client},
    },
    utils,
};
use pb::auth::email_verify::v1::{VerifyRequest, VerifyResponse};

const TOKEN_LEN: usize = 20;

pub fn generate_token() -> String {
    utils::generate_random_string(TOKEN_LEN)
}

pub async fn email_verify(
    server: &AuthServiceProvider<impl EmailSender>,
    request: Request<VerifyRequest>,
) -> Result<Response<VerifyStream>, tonic::Status> {
    let request = request.into_inner();
    let notifier = Arc::new(Notify::new());
    if let Err(e) = verify_client(
        &server.db,
        server.shared_data.clone(),
        VerifyRecord::new(request.email, generate_token()),
        notifier.clone(),
    )
    .await
    {
        tracing::error!("Failed to create verify process:{}", e);
        return Err(tonic::Status::internal("failed to verify"));
    }
    let (tx, rx) = mpsc::channel(1);
    tokio::spawn(async move {
        let expire_timer = async { tokio::time::sleep(VERIFY_EMAIL_EXPIRE).await };
        let ret = select! {
            _ = expire_timer => {
                Err(tonic::Status::deadline_exceeded("Verification has not been down yet"))
            },
            _ = notifier.notified() => {
                Ok(VerifyResponse {})
            }
        };
        tx.send(ret).await.ok();
    });
    let output_stream = ReceiverStream::new(rx);
    Ok(Response::new(Box::pin(output_stream) as VerifyStream))
}
