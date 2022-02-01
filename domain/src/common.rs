use serde::{Deserialize, Serialize};
use std::ops::Deref;
use time::OffsetDateTime;
pub type RuntimeException = Box<dyn std::error::Error>; // because java is better

#[derive(PartialOrd, PartialEq, Debug, Serialize, Deserialize)]
pub struct Id(String);
impl Deref for Id {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Default for Id {
    fn default() -> Id {
        Id(uuid::Uuid::new_v4().to_string())
    }
}

#[derive(PartialOrd, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Metadata {
    id: Id,
    version: Option<u32>,
    creation_date: Option<OffsetDateTime>,
    updated_date: Option<OffsetDateTime>,
}
impl Metadata {
    pub fn id(&self) -> &Id {
        &self.id
    }
    pub fn version(&self) -> &Option<u32> {
        &self.version
    }
    pub fn creation_date(&self) -> &Option<OffsetDateTime> {
        &self.creation_date
    }
    pub fn updated_date(&self) -> &Option<OffsetDateTime> {
        &self.updated_date
    }
    pub fn update_metadata(&mut self) {
        if let Some(version) = self.version {
            self.version = Some(version + 1);
        } else {
            self.version = Some(1);
        }
        if self.creation_date.is_some() {
            self.updated_date = Some(OffsetDateTime::now_utc());
        } else {
            self.creation_date = Some(OffsetDateTime::now_utc());
        }
    }
}

pub trait WithMetadata {
    fn domain_metadata_mut(&mut self) -> &mut Metadata;
}
pub trait WithJsonProcessor<'a, T> {
    fn to_json(&self) -> Result<String, RuntimeException>;
    fn from_json(s: &'a str) -> Result<T, RuntimeException>;
}
impl<'a, T> WithJsonProcessor<'a, T> for T
where
    T: Serialize + Deserialize<'a>,
{
    fn to_json(&self) -> Result<String, RuntimeException> {
        let json = serde_json::to_string(self)?;
        Ok(json)
    }
    fn from_json(s: &'a str) -> Result<T, RuntimeException> {
        let obj: T = serde_json::from_str(s)?;
        Ok(obj)
    }
}

#[cfg(test)]
mod tests {

    use crate::common::{Metadata, WithJsonProcessor};

    use crate::common::Id;

    #[test]
    fn test_uuid_creation() {
        let id: Id = Default::default();
        assert!(!id.is_empty());
    }

    #[test]
    fn test_metadata() {
        let mut metadata: Metadata = Default::default();
        assert_eq!(&None, metadata.version());
        assert_eq!(&None, metadata.updated_date());
        assert_eq!(&None, metadata.creation_date());
        metadata.update_metadata();

        assert_eq!(&Some(1), metadata.version());
        assert_eq!(&None, metadata.updated_date());
        assert!(metadata.creation_date().is_some());
        metadata.update_metadata();

        assert_eq!(&Some(2), metadata.version());
        assert!(metadata.updated_date().is_some());
        assert!(metadata.creation_date().is_some());
        let m = metadata.to_json();
        println!("{}", m.unwrap());
    }
}
