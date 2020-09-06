extern crate toml;

use std::error::Error;
use std::fs::File;
use std::io::Read;

use lazy_static::lazy_static;
use serde::{de, Deserialize, Serialize};

use crate::error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DatabaseConfig {
    pub(crate) name: Option<String>,
    pub(crate) ip: Option<String>,
    pub(crate) port: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmailSenderConfig {
    pub(crate) smtp_server: Option<String>,
    pub(crate) smtp_account: Option<String>,
    pub(crate) smtp_password: Option<String>,
    pub(crate) smtp_port: Option<u16>,
}

lazy_static! {
    pub static ref DEFAULT_DATABASE_CONFIG: DatabaseConfig = sync_new("config/Database.toml")
        .expect("at least one default database config is needed");
    pub static ref DEFAULT_EMAIL_SENDER_CONFIG: EmailSenderConfig = sync_new("config/EmailSender.toml")
        .expect("at least one default email config is needed");
}

pub fn sync_new<T>(filepath: &str) -> Result<T, Box<dyn Error>>
    where T: de::DeserializeOwned
{
    let mut config_str = String::new();
    File::open(filepath)?
        .read_to_string(&mut config_str)?;
    error!(toml::from_str(&config_str))
}

pub async fn new<T>(filepath: &str) -> Result<T, Box<dyn Error>>
    where T: de::DeserializeOwned
{
    let mut config_str = String::new();
    File::open(filepath)?
        .read_to_string(&mut config_str)?;
    error!(toml::from_str(&config_str))
}
#[cfg(test)]
mod test {
    use futures_await_test::async_test;

    use crate::util::config::{DatabaseConfig, EmailSenderConfig, new};

    #[async_test]
    async fn test_load_email_sender_config() {
        let config = new::<EmailSenderConfig>("config/EmailSender.toml").await;
        if let Ok(EmailSenderConfig { smtp_server: Some(smtp_server), smtp_account: Some(smtp_account), smtp_password: Some(_), smtp_port: Some(smtp_port) }) = config {
            assert_eq!(smtp_server, "smtpdm.aliyun.com");
            assert_eq!(smtp_account, "regsiterlink@auto.sustechflow.top");
            assert_eq!(smtp_port, 80);
        } else {
            panic!("fields are missing, failed")
        }
    }

    #[async_test]
    async fn test_load_database_config() {
        let config = new::<DatabaseConfig>("config/DatabaseTest.toml").await;
        if let Ok(DatabaseConfig { name: Some(name), ip: Some(ip), port: Some(port) }) = config {
            assert_eq!(name, "Test");
            assert_eq!(ip, "127.0.0.1");
            assert_eq!(port, 42);
        } else {
            panic!("fields are missing, failed")
        }
    }

    #[async_test]
    async fn test_load_default_database_config() {
        let config = new::<DatabaseConfig>("config/Database.toml").await;
        if let Ok(DatabaseConfig { name: Some(name), ip: Some(ip), port: Some(port) }) = config {
            assert_eq!(name, "SUSTechFlow");
            assert_eq!(ip, "127.0.0.1");
            assert_eq!(port, 27017);
        } else {
            panic!("fields are missing, failed")
        }
    }
}
