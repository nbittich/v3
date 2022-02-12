use tracing::info;

use crate::doc;

use crate::{Client, ClientOptions, Database};
use std::env::var;

const MONGO_HOST: &str = "MONGO_HOST";
const MONGO_PORT: &str = "MONGO_PORT";
const MONGO_USERNAME: &str = "MONGO_USERNAME";
const MONGO_PASSWORD: &str = "MONGO_PASSWORD";
const MONGO_ADMIN_DATABASE: &str = "MONGO_ADMIN_DATABASE";

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

#[cfg(test)]
mod test {
    use crate::doc;
    use futures_util::TryStreamExt;
    use tracing::Level;
    use tracing_subscriber::FmtSubscriber;

    use crate::StoreClient;
    use domain::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Book {
        title: String,
        author: String,
    }

    #[tokio::test]
    async fn test_connection() {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");

        let store_client = StoreClient::new(String::from("test")).await.unwrap();
        let db = store_client.get_db();
        let collection = db.collection::<Book>("books");
        collection.delete_many(doc! {}, None).await.unwrap();
        let books = vec![
            Book {
                title: "The Grapes of Wrath".to_string(),
                author: "John Steinbeck".to_string(),
            },
            Book {
                title: "To Kill a Mockingbird".to_string(),
                author: "Harper Lee".to_string(),
            },
        ];
        collection.insert_many(books, None).await.unwrap();
        let mut cursor_books = collection.find(doc! {}, None).await.unwrap();
        while let Some(book) = cursor_books.try_next().await.unwrap() {
            println!("{}", book.title)
        }
    }
}
