use std::error::Error;

use async_std::task::block_on;
use futures_await_test::async_test;
use lazy_static::lazy_static;
use mongodb::Client;

use crate::{error, util::{config::DatabaseConfig}};

#[derive(Debug)]
pub struct Database {
    pub(crate) name: String,
    pub(crate) ip: String,
    pub(crate) port: u16,
    pub(crate) cli: Client,
}

lazy_static! {
    pub static ref DEFAULT_DATABASE: Database = Database::sync_new(None)
        .expect("at least one usable database is needed");
}

impl DatabaseConfig {
    pub async fn connect(&self) -> Result<Client, Box<dyn Error>> {
        error!(Client::with_uri_str(format!("mongodb://{}:{}", self.ip.as_ref().unwrap(), self.port.as_ref().unwrap()).as_str()).await)
    }
}

impl Database {
    pub async fn new(config: Option<&DatabaseConfig>) -> Result<Database, Box<dyn Error>> {
        use crate::util::config::DEFAULT_DATABASE_CONFIG;
        let config = config.unwrap_or(&*DEFAULT_DATABASE_CONFIG);
        Ok(Database {
            name: config.name.as_ref().ok_or("name is missing")?.clone(),
            ip: config.ip.as_ref().ok_or("ip is missing")?.clone(),
            port: config.port.as_ref().ok_or("port is missing")?.clone(),
            cli: config.connect().await?
        })
    }

    pub fn sync_new(config: Option<&DatabaseConfig>) -> Result<Database, Box<dyn Error>> {
        block_on(Database::new(config))
    }
}

#[async_test]
async fn test_database_connect() {
    let db = Database::new(None).await.unwrap();
    for _ in 1..1000 {
        let cli = &db.cli;
        assert!(cli.list_database_names(None, None).await.unwrap().contains(&db.name));
        cli.database(&db.name);
    }
}
