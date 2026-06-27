use crate::admin_models::{BulkStatusRequest, BulkStatusResponse};
use crate::client::SynapseClient;
use crate::error::SynapseError;

/// Entry point for admin-only Synapse API operations.
///
/// Obtain via [`SynapseClient::admin`]. Requires an API key with admin scope.
pub struct AdminClient<'a> {
    pub(crate) client: &'a SynapseClient,
}

impl<'a> AdminClient<'a> {
    /// Update the status of multiple transactions in one request.
    ///
    /// Calls `PATCH /admin/transactions/bulk-status`. Per-ID outcomes are
    /// always reported individually in [`BulkStatusResponse::errors`] — a
    /// partial failure (some IDs succeed, some don't) is never collapsed
    /// into one opaque top-level error. Check `response.failed` and iterate
    /// `response.errors` to find out exactly which IDs failed and why.
    ///
    /// # Errors
    /// Returns `Err` only for request-level failures (network error, or the
    /// whole request rejected — e.g. bad auth, empty `ids`, or an invalid
    /// `new_status`). Per-ID failures among otherwise-valid IDs are reported
    /// in `Ok(response).errors`, not here.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use synapse_sdk::SynapseClient;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let client = SynapseClient::builder("https://api.example.com", "admin-key").build();
    ///
    /// let ids = vec![
    ///     "550e8400-e29b-41d4-a716-446655440000".to_string(),
    ///     "00000000-0000-0000-0000-000000000000".to_string(),
    /// ];
    ///
    /// match client.admin().bulk_update_status(&ids, "completed").await {
    ///     Ok(response) => {
    ///         println!("{} updated, {} failed", response.updated, response.failed);
    ///         // Partial failure: each failing ID is reported individually,
    ///         // never collapsed into one opaque error for the whole batch.
    ///         for err in &response.errors {
    ///             eprintln!("  {} failed: {}", err.transaction_id, err.error);
    ///         }
    ///     }
    ///     Err(e) => eprintln!("request failed entirely: {}", e),
    /// }
    /// # }
    /// ```
    pub async fn bulk_update_status(
        &self,
        ids: &[String],
        new_status: &str,
    ) -> Result<BulkStatusResponse, SynapseError> {
        let body = BulkStatusRequest {
            transaction_ids: ids,
            status: new_status,
        };
        self.client
            .patch("/admin/transactions/bulk-status", &body)
            .await
    }
}
