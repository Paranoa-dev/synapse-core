pub mod admin;
pub mod admin_models;
pub mod client;
pub mod error;
pub mod pagination;
pub mod retry;

pub use admin::AdminClient;
pub use client::SynapseClient;
pub use error::SynapseError;
