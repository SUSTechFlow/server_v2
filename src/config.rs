extern crate toml;

use std::convert::TryFrom;
use std::error::Error;
use std::fs::File;
use std::io::Read;

use futures_await_test::async_test;
use serde::{Deserialize, Serialize};

use lazy_static::lazy_static;

use crate::error;

#[derive(Serialize, Deserialize, Debug)]
pub struct DatabaseConfig {
    pub(crate) name: Option<String>,
    pub(crate) ip: Option<String>,
    pub(crate) port: Option<u16>,
}

lazy_static! {
    pub static ref DEFAULT_DATABASE_CONFIG: DatabaseConfig = DatabaseConfig::sync_new(None).expect("at least one default database config is needed");
}

impl DatabaseConfig {
    pub fn sync_new(filepath: Option<&str>) -> Result<DatabaseConfig, Box<dyn Error>> {
        let filepath = filepath.unwrap_or("config/Database.toml");
        let mut config_str = String::new();
        File::open(filepath)?
            .read_to_string(&mut config_str)?;
        error!(toml::from_str(&config_str))
    }
    pub async fn new(filepath: Option<&str>) -> Result<DatabaseConfig, Box<dyn Error>> {
        let filepath = filepath.unwrap_or("config/Database.toml");
        let mut config_str = String::new();
        File::open(filepath)?
            .read_to_string(&mut config_str)?;
        error!(toml::from_str(&config_str))
    }
}

#[async_test]
async fn test_load_database_config() {
    let config = DatabaseConfig::new(Some("config/DatabaseTest.toml")).await;
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
    let config = DatabaseConfig::new(None).await;
    if let Ok(DatabaseConfig { name: Some(name), ip: Some(ip), port: Some(port) }) = config {
        assert_eq!(name, "SUSTechFlow");
        assert_eq!(ip, "127.0.0.1");
        assert_eq!(port, 27017);
    } else {
        panic!("fields are missing, failed")
    }
}