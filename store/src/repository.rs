use crate::{doc, to_document, Collection, Document};
use crate::{Cursor, DeleteResult, FindOptions, InsertManyResult};
use domain::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
#[derive(Serialize, Deserialize)]
pub struct Page {
    page: i64,
    limit: i64,
    sort: Option<Document>,
}
pub struct MongoRepository<T: Serialize + DeserializeOwned + Unpin + Send + Sync> {
    collection: Collection<T>,
}

impl<T> MongoRepository<T>
where
    T: Serialize + DeserializeOwned + Unpin + Send + Sync,
{
    pub fn new(collection: Collection<T>) -> Self {
        MongoRepository { collection }
    }
}

impl<T> Repository<T> for MongoRepository<T>
where
    T: Serialize + DeserializeOwned + Unpin + Send + Sync,
{
    fn get_collection(&self) -> &Collection<T> {
        &self.collection
    }
}

#[async_trait::async_trait]
pub trait Repository<T: Serialize + DeserializeOwned + Unpin + Send + Sync> {
    fn get_collection(&self) -> &Collection<T>;
    async fn find_all(&self) -> Result<Cursor<T>, Box<dyn std::error::Error>> {
        let collection = self.get_collection();
        let cursor = collection.find(doc! {}, None).await?;
        Ok(cursor)
    }
    async fn count(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let collection = self.get_collection();
        let count = collection.count_documents(doc! {}, None).await?;
        Ok(count)
    }
    async fn find_page(
        &self,
        query: Option<Document>,
        page: Page,
    ) -> Result<Option<Cursor<T>>, Box<dyn std::error::Error>> {
        let collection = self.get_collection();
        let query = if let Some(q) = query {
            q
        } else {
            doc! {}
        };
        let count = self.count().await? as i64;
        let skip = page.limit * (page.page - 1); // start at page 1
        if count <= skip {
            return Ok(None);
        }
        let options = FindOptions::builder()
            .skip(Some(skip as u64))
            .sort(page.sort)
            .limit(Some(page.limit))
            .build();
        let cursor = collection.find(query, Some(options)).await?;
        Ok(Some(cursor))
    }

    async fn delete_many(
        &self,
        query: Option<Document>,
    ) -> Result<DeleteResult, Box<dyn std::error::Error>> {
        let query = if let Some(q) = query {
            q
        } else {
            doc! {}
        };
        let res = self.get_collection().delete_many(query, None).await?;
        Ok(res)
    }

    async fn insert_many(
        &self,
        data: &Vec<T>,
    ) -> Result<InsertManyResult, Box<dyn std::error::Error>> {
        let res = self.get_collection().insert_many(data, None).await?;
        Ok(res)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<T>, Box<dyn std::error::Error>> {
        let collection = self.get_collection();
        let res = collection.find_one(doc! {"_id": id}, None).await?;
        Ok(res)
    }
    async fn delete_by_id(&self, id: String) -> Result<Option<T>, Box<dyn std::error::Error>> {
        let collection = self.get_collection();
        let res = collection
            .find_one_and_delete(doc! {"_id": id}, None)
            .await?;
        Ok(res)
    }

    async fn update(
        &self,
        id: String,
        entity: &T,
    ) -> Result<Option<T>, Box<dyn std::error::Error>> {
        let collection = self.get_collection();
        let update_doc = to_document(entity)?;
        let res = collection
            .find_one_and_update(doc! {"_id": id}, update_doc, None)
            .await?;
        Ok(res)
    }
}
