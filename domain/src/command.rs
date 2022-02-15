use crate::{Metadata, WithJsonProcessor, WithMetadata};
use serde::{Deserialize, Serialize};

#[derive(PartialOrd, PartialEq, Debug, Serialize, Deserialize, WithMetadata, WithJsonProcessor)]
pub struct CreateUserCommand {
    pub domain_metadata: Metadata,
    pub nickname: String,
    pub password: String,
    pub confirm_password: String,
    pub email: String,
}
