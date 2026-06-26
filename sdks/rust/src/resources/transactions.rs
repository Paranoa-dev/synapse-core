use crate::client::SynapseClient;
use crate::error::SynapseError;
use crate::models::{ListParams, Transaction, TransactionList};

pub struct Transactions<'a> {
    pub(crate) client: &'a SynapseClient,
}

impl<'a> Transactions<'a> {
    /// Fetch a single transaction by its UUID.
    ///
    /// Returns [`SynapseError::NotFound`] when the ID does not exist so callers
    /// can distinguish a missing record from other failure modes without
    /// inspecting raw HTTP status codes.
    ///
    /// # Errors
    /// - [`SynapseError::NotFound`] – no transaction with this ID exists (HTTP 404).
    /// - [`SynapseError::Api`] – server returned another non-success status.
    /// - [`SynapseError::Http`] – network error.
    /// - [`SynapseError::Decode`] – response body is not valid JSON.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use synapse_sdk::{SynapseClient, SynapseError};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let client = SynapseClient::new("https://api.example.com", "your-api-key");
    ///
    /// match client.transactions().get("550e8400-e29b-41d4-a716-446655440000").await {
    ///     Ok(tx) => println!("status: {}", tx.status),
    ///     Err(SynapseError::NotFound(msg)) => eprintln!("not found: {}", msg),
    ///     Err(e) => eprintln!("error: {}", e),
    /// }
    /// # }
    /// ```
    pub async fn get(&self, id: &str) -> Result<Transaction, SynapseError> {
        let path = format!("/transactions/{}", id);
        match self.client.get::<Transaction>(&path).await {
            Err(SynapseError::Api { status: 404, message }) => {
                Err(SynapseError::NotFound(message))
            }
            other => other,
        }
    }

    /// List transactions with optional cursor-based pagination and date filters.
    ///
    /// Pass an [`ListParams`] value to control which page to retrieve. All
    /// fields are optional; omit them to use server defaults (25 records,
    /// forward order, no date filter).
    ///
    /// Cursors are opaque — always use `meta.next_cursor` from a previous
    /// response. Never construct or modify a cursor manually; an invalid or
    /// expired cursor returns [`SynapseError::InvalidCursor`] and must not be
    /// retried as-is.
    ///
    /// # Errors
    /// - [`SynapseError::InvalidCursor`] – the cursor is malformed or expired (HTTP 400).
    /// - [`SynapseError::Api`] – server returned another non-success status.
    /// - [`SynapseError::Http`] – network error.
    /// - [`SynapseError::Decode`] – response body is not valid JSON.
    pub async fn list(&self, params: ListParams) -> Result<TransactionList, SynapseError> {
        let limit_str = params.limit.map(|l| l.to_string());
        let mut query: Vec<(&str, &str)> = Vec::new();

        if let Some(ref v) = params.cursor {
            query.push(("cursor", v.as_str()));
        }
        if let Some(ref v) = limit_str {
            query.push(("limit", v.as_str()));
        }
        if let Some(ref v) = params.from_date {
            query.push(("from_date", v.as_str()));
        }
        if let Some(ref v) = params.to_date {
            query.push(("to_date", v.as_str()));
        }

        match self.client.get_query::<TransactionList>("/transactions", &query).await {
            Err(SynapseError::Api { status: 400, message }) if message.contains("cursor") => {
                Err(SynapseError::InvalidCursor(message))
            }
            other => other,
        }
    }
}
