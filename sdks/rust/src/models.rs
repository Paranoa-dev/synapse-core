use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single transaction returned by the API.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Transaction {
    pub id: String,
    pub stellar_account: String,
    pub amount: String,
    pub asset_code: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub anchor_transaction_id: Option<String>,
    pub callback_type: Option<String>,
    pub callback_status: Option<String>,
    pub settlement_id: Option<String>,
    pub memo: Option<String>,
    pub memo_type: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Pagination metadata included in list responses.
#[derive(Debug, Clone, Deserialize)]
pub struct ListMeta {
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

/// Paginated list of transactions.
#[derive(Debug, Clone, Deserialize)]
pub struct TransactionList {
    pub data: Vec<Transaction>,
    pub meta: ListMeta,
}

/// Filters for [`Transactions::search`].
///
/// All fields are optional; omit a field to leave that dimension unfiltered.
/// A search with no matches returns an empty [`TransactionSearch`] (a page with
/// `total == 0` and no `results`), never an error.
#[derive(Debug, Default)]
pub struct SearchParams {
    /// Exact transaction status (e.g. `"pending"`, `"completed"`).
    pub status: Option<String>,
    /// Exact asset code (e.g. `"USD"`).
    pub asset_code: Option<String>,
    /// Inclusive minimum amount, as a decimal string (e.g. `"10.00"`).
    pub min_amount: Option<String>,
    /// Inclusive maximum amount, as a decimal string (e.g. `"500.00"`).
    pub max_amount: Option<String>,
    /// Inclusive RFC 3339 range start (e.g. `"2024-01-01T00:00:00Z"`).
    pub from: Option<String>,
    /// Exclusive RFC 3339 range end (e.g. `"2024-02-01T00:00:00Z"`).
    pub to: Option<String>,
    /// Exact Stellar account to filter by.
    pub stellar_account: Option<String>,
    /// Opaque pagination cursor from a previous response's `next_cursor`.
    pub cursor: Option<String>,
    /// Maximum records per page (server default: 25, max: 100).
    pub limit: Option<i64>,
}

/// A single page of transactions returned by [`Transactions::search`].
#[derive(Debug, Clone, Deserialize)]
pub struct TransactionSearch {
    /// Total number of records matching the filters across all pages.
    pub total: i64,
    /// Matching transactions for this page (empty when nothing matched).
    #[serde(default)]
    pub results: Vec<Transaction>,
    /// Opaque cursor for the next page, or `None` when this is the last page.
    #[serde(default)]
    pub next_cursor: Option<String>,
}

/// Query parameters for [`Transactions::list`].
///
/// All fields are optional; omit a field to accept the server's default.
/// Never construct a `cursor` manually — always use one from a previous
/// response's `meta.next_cursor`.
#[derive(Debug, Default)]
pub struct ListParams {
    /// Opaque pagination cursor from `meta.next_cursor`.
    pub cursor: Option<String>,
    /// Maximum records per page (server default: 25, max: 100).
    pub limit: Option<i64>,
    /// Inclusive ISO 8601 range start (e.g. `"2024-01-01T00:00:00Z"`).
    pub from_date: Option<String>,
    /// Exclusive ISO 8601 range end (e.g. `"2024-02-01T00:00:00Z"`).
    pub to_date: Option<String>,
}

// ============================================================================
// Admin: Reconciliation Models
// ============================================================================

/// A reconciliation report summary returned by list or run operations.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReconciliationReportSummary {
    pub id: Uuid,
    pub generated_at: DateTime<Utc>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_db_transactions: i32,
    pub total_chain_payments: i32,
    pub missing_on_chain_count: i32,
    pub orphaned_payments_count: i32,
    pub amount_mismatches_count: i32,
    pub has_discrepancies: bool,
}

/// Paginated list of reconciliation reports.
#[derive(Debug, Clone, Deserialize)]
pub struct ListReconciliationReports {
    pub reports: Vec<ReconciliationReportSummary>,
    pub total: i64,
    pub limit: i32,
    pub offset: i32,
}

/// Query parameters for listing reconciliation reports.
#[derive(Debug, Default)]
pub struct ListReportsParams {
    /// Maximum records per page (server default: 20).
    pub limit: Option<i32>,
    /// Number of records to skip.
    pub offset: Option<i32>,
}

/// A missing transaction detail in a reconciliation report.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MissingTransaction {
    pub id: Uuid,
    pub stellar_account: String,
    pub amount: String,
    pub asset_code: String,
    pub memo: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// An orphaned payment detail in a reconciliation report.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrphanedPayment {
    pub payment_id: String,
    pub from: String,
    pub to: String,
    pub amount: String,
    pub asset_code: String,
    pub memo: Option<String>,
}

/// An amount mismatch detail in a reconciliation report.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AmountMismatch {
    pub transaction_id: Uuid,
    pub payment_id: String,
    pub db_amount: String,
    pub chain_amount: String,
    pub memo: Option<String>,
}

/// Full reconciliation report details.
#[derive(Debug, Clone, Deserialize)]
pub struct ReconciliationReportDetail {
    pub id: Uuid,
    pub generated_at: DateTime<Utc>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub summary: ReconciliationSummary,
    pub missing_on_chain: Vec<MissingTransaction>,
    pub orphaned_payments: Vec<OrphanedPayment>,
    pub amount_mismatches: Vec<AmountMismatch>,
}

/// Summary statistics in a full reconciliation report.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReconciliationSummary {
    pub total_db_transactions: usize,
    pub total_chain_payments: usize,
    pub missing_on_chain_count: i32,
    pub orphaned_payments_count: i32,
    pub amount_mismatches_count: i32,
    pub has_discrepancies: bool,
}

/// Response from running a reconciliation.
#[derive(Debug, Clone, Deserialize)]
pub struct RunReconciliationResponse {
    pub message: String,
    pub report: ReconciliationReportSummary,
}

/// Request to run a reconciliation.
#[derive(Debug, Serialize)]
pub struct RunReconciliationRequest {
    pub account: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period_hours: Option<i32>,
}
