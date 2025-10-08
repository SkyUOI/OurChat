use crate::consts;
use crate::setting::read_config_and_deserialize;
use anyhow::bail;
use lettre::message::header::ContentType;
use lettre::message::{Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug)]
pub struct EmailClient {
    client: InternalEmailClient,
    from: Mailbox,
}

type InternalEmailClient = AsyncSmtpTransport<lettre::Tokio1Executor>;

impl EmailClient {
    pub fn new(client: InternalEmailClient, email_address: &str) -> anyhow::Result<Self> {
        let mailbox = format!("{} <{}>", consts::APP_NAME, email_address).parse()?;
        Ok(EmailClient {
            client,
            from: mailbox,
        })
    }
}

#[mockall::automock]
#[async_trait::async_trait]
pub trait EmailSender: Send + Sync + std::fmt::Debug + 'static {
    async fn send(
        &self,
        to: Mailbox,
        subject: String,
        text_body: String,
        html_body: Option<String>,
    ) -> anyhow::Result<()>;

    async fn send_low(&self, msg: lettre::Message) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
impl EmailSender for EmailClient {
    async fn send(
        &self,
        to: Mailbox,
        subject: String,
        text_body: String,
        html_body: Option<String>,
    ) -> anyhow::Result<()> {
        let mut multipart = MultiPart::alternative().singlepart(
            SinglePart::builder()
                .header(ContentType::TEXT_PLAIN)
                .body(text_body),
        );

        if let Some(html) = html_body {
            multipart = multipart.singlepart(
                SinglePart::builder()
                    .header(ContentType::TEXT_HTML)
                    .body(html_body),
            )
        };

        let email = lettre::Message::builder()
            .from(self.from.clone())
            .to(to.clone())
            .subject(subject)
            .multipart(multipart);

        self.send_low(email).await?;
        Ok(())
    }

    async fn send_low(&self, msg: lettre::Message) -> anyhow::Result<()> {
        self.client.send(msg).await?;
        Ok(())
    }
}

impl EmailCfg {
    pub fn email_available(&self) -> bool {
        self.email_address.is_some() && self.smtp_address.is_some() && self.smtp_password.is_some()
    }

    pub fn build_email_client(&self) -> anyhow::Result<EmailClient> {
        if !self.email_available() {
            bail!("email is not available");
        }
        let creds = Credentials::new(
            self.email_address.clone().unwrap(),
            self.smtp_password.clone().unwrap(),
        );
        EmailClient::new(
            AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(
                &self.smtp_address.clone().unwrap(),
            )?
            .credentials(creds)
            .build(),
            self.email_address.as_ref().unwrap(),
        )
    }

    pub fn build_from_path(path: &Path) -> anyhow::Result<Self> {
        read_config_and_deserialize(path)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmailCfg {
    #[serde(default)]
    pub email_address: Option<String>,
    #[serde(default)]
    pub smtp_address: Option<String>,
    #[serde(default)]
    pub smtp_password: Option<String>,
}
