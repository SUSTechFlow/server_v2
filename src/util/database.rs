use std::error::Error;

use futures_await_test::async_test;
use mongodb::Client;

use lazy_static::lazy_static;

use crate::{error,util::{config::DatabaseConfig}};

#[derive(Debug)]
pub struct Database {
    pub(crate) name: String,
    pub(crate) ip: String,
    pub(crate) port: u16,
}

lazy_static! {
    pub static ref DEFAULT_DATABASE: Database = Database::sync_new(None).expect("at least one usable database is needed");
}


impl Database {
    pub async fn new(config: Option<&DatabaseConfig>) -> Result<Database, Box<dyn Error>> {
        use crate::util::config::DEFAULT_DATABASE_CONFIG;
        let config = config.unwrap_or(&*DEFAULT_DATABASE_CONFIG);
        Ok(Database {
            name: config.name.as_ref().ok_or("name is missing")?.clone(),
            ip: config.ip.as_ref().ok_or("ip is missing")?.clone(),
            port: config.port.as_ref().ok_or("port is missing")?.clone(),
        })
    }

    pub fn sync_new(config: Option<&DatabaseConfig>) -> Result<Database, Box<dyn Error>> {
        use crate::util::config::DEFAULT_DATABASE_CONFIG;
        let config = config.unwrap_or(&*DEFAULT_DATABASE_CONFIG);
        Ok(Database {
            name: config.name.as_ref().ok_or("name is missing")?.clone(),
            ip: config.ip.as_ref().ok_or("ip is missing")?.clone(),
            port: config.port.as_ref().ok_or("port is missing")?.clone(),
        })
    }

    pub async fn connect(&self) -> Result<Client, Box<dyn Error>> {
        error!(Client::with_uri_str(format!("mongodb://{}:{}", self.ip, self.port).as_str()).await)
    }
}

#[async_test]
async fn test_database_connect() {
    let db = Database::new(None).await.unwrap();
    let cli = db.connect().await.unwrap();
    assert!(cli.list_database_names(None, None).await.unwrap().contains(&db.name));
    cli.database(&db.name);
}
