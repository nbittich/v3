use tracing::info;

use crate::doc;

use crate::{Client, ClientOptions, Database};
use std::env::var;

const MONGO_HOST: &str = "MONGO_HOST";
const MONGO_PORT: &str = "MONGO_PORT";
const MONGO_USERNAME: &str = "MONGO_USERNAME";
const MONGO_PASSWORD: &str = "MONGO_PASSWORD";
const MONGO_ADMIN_DATABASE: &str = "MONGO_ADMIN_DATABASE";
#[derive(Debug)]
pub struct StoreClient {
    client: Client,
    application_db: String,
}

impl StoreClient {
    pub async fn new(application_name: String) -> Result<StoreClient, Box<dyn std::error::Error>> {
        let client = StoreClient::create_client(application_name.clone()).await?;

        Ok(StoreClient {
            client,
            application_db: application_name,
        })
    }

    pub fn get_client(&self) -> Client {
        self.client.clone()
    }

    pub fn get_db(&self) -> Database {
        let client = self.get_client();
        client.database(&self.application_db)
    }

    #[tracing::instrument]
    async fn create_client(application_name: String) -> Result<Client, Box<dyn std::error::Error>> {
        let mongo_host = var(MONGO_HOST).unwrap_or_else(|_| String::from("127.0.0.1"));
        let mongo_port = var(MONGO_PORT).unwrap_or_else(|_| String::from("27017"));
        let mongo_username = var(MONGO_USERNAME).unwrap_or_else(|_| String::from("root"));
        let mongo_password = var(MONGO_PASSWORD).unwrap_or_else(|_| String::from("root"));
        let mongo_admin_db = var(MONGO_ADMIN_DATABASE).unwrap_or_else(|_| String::from("admin"));
        let mut client_options = ClientOptions::parse(format!(
            "mongodb://{mongo_username}:{mongo_password}@{mongo_host}:{mongo_port}"
        ))
        .await?;
        client_options.app_name = Some(application_name);
        let client = Client::with_options(client_options)?;
        let _ = client
            .database(&mongo_admin_db)
            .run_command(doc! {"ping": 1}, None)
            .await?;
        info!("Successfully connected");
        Ok(client)
    }
}
