use crate::client::SynapseClient;
use crate::error::SynapseError;
use crate::models::{Settlement, SettlementListResponse};

pub struct Settlements<'a> {
    pub(crate) client: &'a SynapseClient,
}

impl<'a> Settlements<'a> {
    /// List settlements with optional cursor-based pagination.
    ///
    /// Pass a [`crate::models::ListParams`] to control pagination. The server
    /// enforces page limits; omitted fields fall back to defaults.
    ///
    /// # Errors
    /// - [`SynapseError::InvalidCursor`] – the cursor is malformed or expired.
    /// - [`SynapseError::Api`] – server returned another non-success status.
    /// - [`SynapseError::Http`] – network error.
    /// - [`SynapseError::Decode`] – response body is not valid JSON.
    pub async fn list(
        &self,
        params: crate::models::ListParams,
    ) -> Result<SettlementListResponse, SynapseError> {
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

        self.client
            .get_query::<SettlementListResponse>("/settlements", &query)
            .await
    }

    /// Fetch a single settlement by its UUID.
    ///
    /// Returns [`SynapseError::NotFound`] when the ID does not exist so callers
    /// can distinguish a missing record from other failure modes.
    ///
    /// # Errors
    /// - [`SynapseError::NotFound`] – no settlement with this ID exists (HTTP 404).
    /// - [`SynapseError::Api`] – server returned another non-success status.
    /// - [`SynapseError::Http`] – network error.
    /// - [`SynapseError::Decode`] – response body is not valid JSON.
    pub async fn get(&self, id: &str) -> Result<Settlement, SynapseError> {
        let path = format!("/settlements/{}", id);
        match self.client.get::<Settlement>(&path).await {
            Err(SynapseError::Http {
                status: 404,
                body,
            }) => Err(SynapseError::NotFound(body)),
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn settlement_body(id: &str) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "asset_code": "USD",
            "total_amount": "5000.00",
            "tx_count": 42,
            "period_start": "2024-01-01T00:00:00Z",
            "period_end": "2024-01-31T23:59:59Z",
            "status": "completed",
            "created_at": "2024-02-01T10:00:00Z",
            "updated_at": "2024-02-01T10:00:00Z",
            "dispute_reason": null,
            "original_total_amount": null,
            "reviewed_by": null,
            "reviewed_at": null
        })
    }

    #[tokio::test]
    async fn get_returns_settlement_on_200() {
        let server = MockServer::start().await;
        let settlement_id = "550e8400-e29b-41d4-a716-446655440000";

        Mock::given(method("GET"))
            .and(path(format!("/settlements/{}", settlement_id)))
            .and(header("X-API-Key", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(settlement_body(settlement_id)))
            .mount(&server)
            .await;

        let client = SynapseClient::new(server.uri(), "test-key");
        let result = client.settlements().get(settlement_id).await;

        assert!(result.is_ok(), "expected Ok, got: {:?}", result);
        let settlement = result.unwrap();
        assert_eq!(settlement.id, settlement_id);
        assert_eq!(settlement.asset_code, "USD");
        assert_eq!(settlement.status, "completed");
    }

    #[tokio::test]
    async fn get_returns_not_found_on_404() {
        let server = MockServer::start().await;
        let settlement_id = "00000000-0000-0000-0000-000000000000";

        Mock::given(method("GET"))
            .and(path(format!("/settlements/{}", settlement_id)))
            .respond_with(
                ResponseTemplate::new(404).set_body_string("Settlement not found"),
            )
            .mount(&server)
            .await;

        let client = SynapseClient::new(server.uri(), "test-key");
        let result = client.settlements().get(settlement_id).await;

        assert!(
            matches!(result, Err(SynapseError::NotFound(_))),
            "expected NotFound, got: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn list_returns_page_on_200() {
        let server = MockServer::start().await;
        let settlement_id = "550e8400-e29b-41d4-a716-446655440000";

        Mock::given(method("GET"))
            .and(path("/settlements"))
            .and(header("X-API-Key", "test-key"))
            .and(query_param("limit", "10"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "settlements": [settlement_body(settlement_id)],
                "next_cursor": "next-page-token",
                "has_more": true
            })))
            .mount(&server)
            .await;

        let client = SynapseClient::new(server.uri(), "test-key");
        let params = crate::models::ListParams {
            limit: Some(10),
            ..Default::default()
        };
        let result = client.settlements().list(params).await;

        assert!(result.is_ok(), "expected Ok, got: {:?}", result);
        let page = result.unwrap();
        assert_eq!(page.settlements.len(), 1);
        assert_eq!(page.settlements[0].id, settlement_id);
        assert_eq!(page.next_cursor.as_deref(), Some("next-page-token"));
        assert!(page.has_more);
    }

    #[tokio::test]
    async fn list_returns_empty_page_on_zero_matches() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/settlements"))
            .and(query_param("cursor", "nonexistent"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "settlements": [],
                "next_cursor": null,
                "has_more": false
            })))
            .mount(&server)
            .await;

        let client = SynapseClient::new(server.uri(), "test-key");
        let params = crate::models::ListParams {
            cursor: Some("nonexistent".to_string()),
            ..Default::default()
        };
        let result = client.settlements().list(params).await;

        assert!(
            result.is_ok(),
            "zero matches must be an empty page, not an error: {:?}",
            result
        );
        let page = result.unwrap();
        assert_eq!(page.settlements.len(), 0);
        assert!(page.next_cursor.is_none());
        assert!(!page.has_more);
    }
}
