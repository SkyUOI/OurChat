use crate::consts;
use lettre::message::Mailbox;
use lettre::{AsyncSmtpTransport, AsyncTransport};

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
    async fn send<T: Into<String> + Send + 'static>(
        &self,
        to: Mailbox,
        subject: T,
        body: String,
    ) -> anyhow::Result<()>;

    async fn send_low(&self, msg: lettre::Message) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
impl EmailSender for EmailClient {
    async fn send<T: Into<String> + Send + 'static>(
        &self,
        to: Mailbox,
        subject: T,
        body: String,
    ) -> anyhow::Result<()> {
        let email = lettre::Message::builder()
            .from(self.from.clone())
            .to(to.clone())
            .subject(subject.into())
            .body(body)?;
        self.send_low(email).await?;
        Ok(())
    }

    async fn send_low(&self, msg: lettre::Message) -> anyhow::Result<()> {
        self.client.send(msg).await?;
        Ok(())
    }
}
