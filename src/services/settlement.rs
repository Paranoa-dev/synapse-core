use crate::db::models::Settlement;
use crate::db::queries;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use bigdecimal::BigDecimal;

/// Maps a `sqlx::Error` to the appropriate `AppError` variant.
///
/// `RowNotFound` is treated as a domain-level not-found rather than a generic
/// database error so callers can distinguish the two cases.
fn map_db_err(e: sqlx::Error) -> AppError {
    match e {
        sqlx::Error::RowNotFound => AppError::NotFound("settlement record not found".to_string()),
        other => AppError::DatabaseError(other.to_string()),
    }
}

pub struct SettlementService {
    pool: PgPool,
}

impl SettlementService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Run settlement for all assets with completed, unsettled transactions.
    pub async fn run_settlements(&self) -> Result<Vec<Settlement>, AppError> {
        let assets = queries::get_unique_assets_to_settle(&self.pool)
            .await
            .map_err(map_db_err)?;

        let mut results = Vec::new();
        for asset in assets {
            match self.settle_asset(&asset).await {
                Ok(Some(settlement)) => results.push(settlement),
                Ok(None) => tracing::info!("No transactions to settle for asset {}", asset),
                Err(e) => tracing::error!("Failed to settle asset {}: {:?}", asset, e),
            }
        }

        Ok(results)
    }

    /// Settle transactions for a specific asset.
    ///
    /// Returns `Ok(None)` when there are no unsettled transactions for the
    /// given asset, `Ok(Some(settlement))` on success, and an `AppError` on
    /// any database or domain-level failure.
    pub async fn settle_asset(&self, asset_code: &str) -> Result<Option<Settlement>, AppError> {
        // Validate asset_code is non-empty before touching the database.
        if asset_code.trim().is_empty() {
            return Err(AppError::InvalidSettlementAmount(
                "asset_code must not be empty".to_string(),
            ));
        }

        let mut tx = self.pool.begin().await.map_err(map_db_err)?;

        // We settle everything up to "now"
        let end_time = Utc::now();

        // Fetch candidate transactions with FOR UPDATE lock
        let unsettled = queries::get_unsettled_transactions(&mut tx, asset_code, end_time)
            .await
            .map_err(|e| {
                // Roll back is best-effort here; the transaction will be
                // aborted by the server when the connection is dropped anyway.
                tracing::warn!("Failed to fetch unsettled transactions for {asset_code}: {e}");
                map_db_err(e)
            })?;

        if unsettled.is_empty() {
            tx.rollback().await.map_err(map_db_err)?;
            return Ok(None);
        }

        let tx_count = unsettled.len() as i32;
        let total_amount: BigDecimal = unsettled
            .iter()
            .map(|t| t.amount.clone())
            .fold(BigDecimal::from(0), |acc, x| acc + x);

        // Reject a settlement whose net amount is zero or negative — this
        // would indicate corrupted data and should never be committed.
        if total_amount <= BigDecimal::from(0) {
            tx.rollback().await.map_err(map_db_err)?;
            return Err(AppError::InvalidSettlementAmount(format!(
                "computed total for asset '{asset_code}' is non-positive: {total_amount}"
            )));
        }

        // Find the range of transactions
        let period_start = unsettled
            .iter()
            .map(|t| t.created_at)
            .min()
            .unwrap_or(end_time);
        let period_end = unsettled
            .iter()
            .map(|t| t.updated_at)
            .max()
            .unwrap_or(end_time);

        let settlement = Settlement {
            id: Uuid::new_v4(),
            asset_code: asset_code.to_string(),
            total_amount,
            tx_count,
            period_start,
            period_end,
            status: "completed".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Save settlement record
        let saved_settlement = queries::insert_settlement(&mut tx, &settlement)
            .await
            .map_err(map_db_err)?;

        // Link transactions to settlement
        let tx_ids: Vec<Uuid> = unsettled.iter().map(|t| t.id).collect();
        queries::update_transactions_settlement(&mut tx, &tx_ids, saved_settlement.id)
            .await
            .map_err(map_db_err)?;

        tx.commit().await.map_err(map_db_err)?;

        // Invalidate cache after successful commit
        queries::invalidate_caches_for_asset(asset_code).await;

        tracing::info!(
            "Settled {} transactions for asset {} (ID: {})",
            tx_count,
            asset_code,
            saved_settlement.id
        );

        Ok(Some(saved_settlement))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_db_err_row_not_found_becomes_not_found() {
        let err = map_db_err(sqlx::Error::RowNotFound);
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[test]
    fn map_db_err_other_becomes_database_error() {
        let err = map_db_err(sqlx::Error::PoolTimedOut);
        assert!(matches!(err, AppError::DatabaseError(_)));
    }
}
