use crate::OffsetDateTime;
use crate::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(PartialOrd, PartialEq, Debug, Clone, Serialize, Deserialize)]
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

impl From<String> for Id {
    fn from(id: String) -> Self {
        Id(id)
    }
}

#[derive(
    PartialOrd, PartialEq, Debug, Default, Serialize, Deserialize, crate::WithJsonProcessor,
)]
pub struct Metadata {
    id: Id,
    version: Option<u32>,
    creation_date: Option<OffsetDateTime>,
    updated_date: Option<OffsetDateTime>,
}
impl Metadata {
    pub fn new_with_default(id: &Id) -> Metadata {
        Metadata {
            id: id.clone(),
            ..Default::default()
        }
    }
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
pub trait WithJsonProcessor<'a> {
    type Output;
    fn to_json(&self) -> anyhow::Result<String>;
    fn to_json_pretty(&self) -> anyhow::Result<String>;
    fn from_json(s: &'a str) -> anyhow::Result<Self::Output>;
    fn from_json_slice(s: &'a [u8]) -> anyhow::Result<Self::Output>;
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
