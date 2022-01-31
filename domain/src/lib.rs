mod common;
mod user;

pub use common::Id;
pub use common::Metadata;
pub use common::RuntimeException;
pub use common::WithJsonProcessor;
pub use common::WithMetadata;
pub use domain_macro::WithMetadataMacro;
pub use user::Address;
pub use user::Profile;
pub use user::User;

#[cfg(test)]
mod tests {}
