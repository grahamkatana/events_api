use async_trait::async_trait;
use lettre::{
    message::header::ContentType, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

// A `trait` defines *behavior* without saying *how* it's done — any
// type that implements `send` this way can be used wherever
// `EmailSender` is expected. This is what makes our email sending
// "agnostic": our register handler will only ever know about this
// trait, never about MailHog or SMTP specifically.
#[async_trait]
pub trait EmailSender: Send + Sync {
    async fn send(&self, to: &str, subject: &str, html_body: String) -> Result<(), String>;
}

pub struct SmtpMailer {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: String,
}

impl SmtpMailer {
    pub fn from_env() -> Self {
        let host = std::env::var("SMTP_HOST").expect("SMTP_HOST must be set");
        let port: u16 = std::env::var("SMTP_PORT")
            .expect("SMTP_PORT must be set")
            .parse()
            .expect("SMTP_PORT must be a valid number");
        let from = std::env::var("SMTP_FROM").expect("SMTP_FROM must be set");

        // `builder_dangerous` builds a connection with NO implicit TLS —
        // fine for MailHog on localhost. A real provider would instead
        // use `AsyncSmtpTransport::relay(&host)` (which enables TLS)
        // and `.credentials(Credentials::new(user, pass))` — everything
        // *else* in this file stays identical either way.
        let transport = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&host)
            .port(port)
            .build();

        SmtpMailer { transport, from }
    }
}

#[async_trait]
impl EmailSender for SmtpMailer {
    async fn send(&self, to: &str, subject: &str, html_body: String) -> Result<(), String> {
        let email = Message::builder()
            .from(self.from.parse().map_err(|e: lettre::address::AddressError| e.to_string())?)
            .to(to.parse().map_err(|e: lettre::address::AddressError| e.to_string())?)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(html_body)
            .map_err(|e| e.to_string())?;

        self.transport
            .send(email)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}