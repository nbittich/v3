#[cfg(test)]
mod test {
    use domain::WithJsonProcessor;
    use futures_util::TryStreamExt;
    use store::{doc, MongoRepository, Repository};
    use tracing::Level;
    use tracing_subscriber::FmtSubscriber;

    use domain::{Deserialize, Id, Serialize};
    use store::StoreClient;

    #[derive(Debug, Default, WithJsonProcessor, Serialize, Deserialize)]
    struct Book {
        #[serde(rename = "_id")]
        id: Id,
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
        let repository = MongoRepository::new(collection);

        repository.delete_many(None).await.unwrap();
        let books = vec![
            Book {
                title: "The Grapes of Wrath".to_string(),
                author: "John Steinbeck".to_string(),
                ..Default::default()
            },
            Book {
                title: "To Kill a Mockingbird".to_string(),
                author: "Harper Lee".to_string(),
                ..Default::default()
            },
        ];
        repository.insert_many(&books).await.unwrap();
        println!("from find by id");
        let book = books.get(0).unwrap();
        let id = &book.id;
        let result = repository.find_by_id(id).await.unwrap();
        if let Some(book) = result {
            println!("found the book {}", book.to_json_pretty().unwrap());
        } else {
            println!("book not found");
        }
        let mut cursor_books = repository.find_all().await.unwrap();
        while let Some(book) = cursor_books.try_next().await.unwrap() {
            println!("{}", book.to_json().unwrap())
        }
    }
}
