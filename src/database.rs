use std::error::Error;
use mongodb::{Client, options::ClientOptions};
use crate::{config::DatabaseConfig, error};
use std::convert::TryFrom;
#[derive(Debug)]
pub struct Database {
    name: String,
    ip: String,
    port: u16,
}

impl Database {
    pub async fn new(config: Option<DatabaseConfig>) -> Result<Database, Box<dyn Error>> {
        let config = match config {
            None => DatabaseConfig::new(None).await?,
            Some(config) => config,
        };
        Ok(Database {
            name: config.name.ok_or("name is missing")?,
            ip: config.ip.ok_or("ip is missing")?,
            port: config.port.ok_or("port is missing")?
        })
    }

    pub async fn connect(&self) -> Result<Client, Box<dyn Error>> {
        let client_option = ClientOptions::parse(format!("mongodb://{}:{}",self.ip, self.port).as_str()).await?;
        error!(Client::with_options(client_option))
    }
}

#[test]
fn test_database_connect() {
    use futures::executor::block_on;
    let db = Database::new(None).unwrap();
    let cli = block_on(db.connect()).unwrap();
    assert!(block_on(cli.list_database_names(None, None)).unwrap().contains(&db.name));
    cli.database(&db.name);
}
