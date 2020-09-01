use std::error::Error;

use async_std::task::block_on;
use futures_await_test::async_test;
use lazy_static::lazy_static;
use lettre::{Message, SmtpTransport, Tls, Transport};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::response::Response;

use crate::util::config::EmailSenderConfig;

pub struct EmailSender {
    pub(crate) smtp_server: String,
    pub(crate) smtp_account: String,
    pub(crate) smtp_port: u16,
    pub(crate) cred: Credentials,
}

lazy_static! {
    static ref DEFAULT_EMAIL_SENDER: EmailSender = EmailSender::sync_new(None)
            .expect("at least one usable email sender is needed");
}

impl EmailSender {
    pub async fn new(config: Option<&EmailSenderConfig>) -> Result<EmailSender, Box<dyn Error>> {
        use crate::util::config::DEFAULT_EMAIL_SENDER_CONFIG;
        let config = config.unwrap_or(&*DEFAULT_EMAIL_SENDER_CONFIG);

        let smtp_account = config.smtp_account.as_ref().ok_or("smtp_account is missing")?.clone();
        let smtp_password = config.smtp_password.as_ref().ok_or("smtp_password is missing")?.clone();
        let smtp_server = config.smtp_server.as_ref().ok_or("smtp_server is missing")?.clone();
        let smtp_port = config.smtp_port.as_ref().ok_or("smtp_port is missing")?.clone();

        let cred = Credentials::new(smtp_account.clone(), smtp_password);

        Ok(EmailSender {
            smtp_server,
            smtp_account,
            smtp_port,
            cred,
        })
    }

    pub fn sync_new(config: Option<&EmailSenderConfig>) -> Result<EmailSender, Box<dyn Error>> {
        block_on(EmailSender::new(config))
    }
}

impl EmailSender {
    pub async fn send(&self, recv_addr: &str, subject: &str, body: &str) -> Result<Response, Box<dyn Error>> {
        let email = Message::builder()
            .from(self.smtp_account.parse()?)
            .to(recv_addr.parse()?)
            .subject(subject)
            .body(body)?;
        let mailer = SmtpTransport::relay(&self.smtp_server)?
            .port(self.smtp_port)
            .credentials(self.cred.clone())
            .tls(Tls::None)
            .build();
        Ok(mailer.send(&email)?)
    }
}

#[async_test]
async fn test_email_send() {
    let sender = EmailSender::new(None).await.unwrap();
    sender.send("11712009@mail.sustech.edu.cn", "test", "test").await.unwrap();
}