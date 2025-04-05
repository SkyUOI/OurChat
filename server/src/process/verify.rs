use crate::{
    helper,
    process::error_msg::SERVER_ERROR,
    server::{AuthServiceProvider, VerifyStream},
};
use anyhow::Context;
use base::rabbitmq::http_server::VerifyRecord;
use deadpool_lapin::lapin::options::{BasicPublishOptions, ConfirmSelectOptions};
use pb::service::auth::email_verify::v1::{VerifyRequest, VerifyResponse};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

const TOKEN_LEN: usize = 20;

pub fn generate_token() -> String {
    helper::generate_random_string(TOKEN_LEN)
}

pub async fn email_verify(
    server: &AuthServiceProvider,
    request: Request<VerifyRequest>,
) -> Result<Response<VerifyStream>, Status> {
    match email_verify_impl(server, request).await {
        Ok(res) => Ok(res),
        Err(e) => match e {
            VerifyError::Db(_) | VerifyError::Internal(_) | VerifyError::Rabbitmq(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            VerifyError::Status(s) => Err(s),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum VerifyError {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
    #[error("rabbitmq:{0:?}")]
    Rabbitmq(#[from] deadpool_lapin::lapin::Error),
}

async fn email_verify_impl(
    server: &AuthServiceProvider,
    request: Request<VerifyRequest>,
) -> Result<Response<VerifyStream>, VerifyError> {
    let request = request.into_inner();
    let connection = server
        .rabbitmq
        .get()
        .await
        .context("Cannot get rabbit connection")?;
    let channel = connection.create_channel().await?;
    let json_record =
        serde_json::to_string(&VerifyRecord::new(request.email.clone(), generate_token()))
            .context("Cannot get json")?;
    channel
        .basic_publish(
            "",
            base::rabbitmq::http_server::VERIFY_QUEUE,
            BasicPublishOptions::default(),
            json_record.as_bytes(),
            Default::default(),
        )
        .await?;
    channel
        .confirm_select(ConfirmSelectOptions::default())
        .await?;

    let (tx, rx) = mpsc::channel(1);
    tokio::spawn(async move {
        let ret = match channel.wait_for_confirms().await {
            Ok(_) => Ok(VerifyResponse {}),
            Err(_) => Err(Status::deadline_exceeded("Verification Failed")),
        };
        tx.send(ret).await.ok();
    });
    let output_stream = ReceiverStream::new(rx);
    Ok(Response::new(Box::pin(output_stream) as VerifyStream))
}
