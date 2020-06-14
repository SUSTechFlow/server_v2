extern crate toml;

use std::error::Error;
use std::fs::File;
use std::io::Read;
use crate::error;
use std::convert::TryFrom;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct DatabaseConfig {
    pub(crate) name: Option<String>,
    pub(crate) ip: Option<String>,
    pub(crate) port: Option<u16>,
}

impl DatabaseConfig {
    pub async fn new(filepath: Option<&str>) -> Result<DatabaseConfig, Box<dyn Error>> {
        let filepath = match filepath {
            None => "config/Database.toml",
            Some(f) => f
        };
        let mut config_str = String::new();
        let file = File::open(filepath);
        file?.read_to_string(&mut config_str)?;
        error!(toml::from_str(&config_str))
    }
}


#[test]
fn test_load_database_config() {
    let config = DatabaseConfig::new(Some("config/DatabaseTest.toml"));
    if let Ok(DatabaseConfig { name: Some(name), ip: Some(ip), port: Some(port) }) = config {
        assert_eq!(name, "Test");
        assert_eq!(ip, "127.0.0.1");
        assert_eq!(port, 42);
    } else {
        panic!("fields are missing, failed")
    }
}

#[test]
fn test_load_default_database_config() {
    let config = DatabaseConfig::new(None);
    if let Ok(DatabaseConfig { name: Some(name), ip: Some(ip), port: Some(port) }) = config {
        assert_eq!(name, "SUSTechFlow");
        assert_eq!(ip, "127.0.0.1");
        assert_eq!(port, 27017);
    } else {
        panic!("fields are missing, failed")
    }
}