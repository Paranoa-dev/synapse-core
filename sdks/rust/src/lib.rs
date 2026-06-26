pub mod client;
pub mod error;
pub mod models;
pub mod pagination;
pub mod resources;
pub mod retry;

pub use client::{SynapseClient, SynapseClientBuilder};
pub use error::SynapseError;
pub use models::{ListParams, SearchParams, Transaction, TransactionList, TransactionSearch};
pub use resources::graphql::GraphQL;
pub use resources::transactions::Transactions;

impl SynapseClient {
    /// Access the `transactions` resource methods.
    pub fn transactions(&self) -> resources::transactions::Transactions<'_> {
        resources::transactions::Transactions { client: self }
    }

    /// Access the `graphql` resource methods.
    pub fn graphql(&self) -> resources::graphql::GraphQL<'_> {
        resources::graphql::GraphQL { client: self }
    }
}
