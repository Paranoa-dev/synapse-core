use serde::{Deserialize, Serialize};

/// Request body for [`crate::admin::AdminClient::bulk_update_status`].
#[derive(Debug, Clone, Serialize)]
pub(crate) struct BulkStatusRequest<'a> {
    pub transaction_ids: &'a [String],
    pub status: &'a str,
}

/// Per-transaction outcome when a bulk status update fails for that ID.
///
/// Mirrors the backend's `BulkUpdateError` (`src/db/queries.rs`).
#[derive(Debug, Clone, Deserialize)]
pub struct BulkUpdateError {
    pub transaction_id: String,
    pub error: String,
}

/// Result of [`crate::admin::AdminClient::bulk_update_status`].
///
/// `updated` and `failed` are counts; `errors` reports exactly which IDs
/// failed and why. A partial failure is never collapsed into one opaque
/// error — always check `errors` rather than inferring failure from `failed`
/// alone, since it also gives you the per-ID reason.
#[derive(Debug, Clone, Deserialize)]
pub struct BulkStatusResponse {
    pub updated: usize,
    pub failed: usize,
    pub errors: Vec<BulkUpdateError>,
}
