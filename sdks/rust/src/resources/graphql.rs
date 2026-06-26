use crate::client::SynapseClient;
use crate::error::SynapseError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub struct GraphQL<'a> {
    pub(crate) client: &'a SynapseClient,
}

/// The raw envelope returned by `POST /graphql`.
///
/// On success `data` is populated. On a GraphQL-level error the server still
/// returns HTTP 200 but populates `errors` instead of (or alongside) `data`.
#[derive(Debug, Deserialize)]
struct GraphqlEnvelope {
    #[serde(default)]
    data: Option<Value>,
    #[serde(default)]
    errors: Option<Vec<Value>>,
}

#[derive(Debug, Clone, Serialize)]
struct GraphqlRequest<'a> {
    query: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<Value>,
}

impl<'a> GraphQL<'a> {
    /// Execute a GraphQL query against `POST /graphql`.
    ///
    /// `variables` may be any JSON-serialisable value (typically a
    /// `serde_json::json!({…})` object), or `None` when the query has no
    /// variables.
    ///
    /// GraphQL errors (HTTP 200 + `"errors"` array) are surfaced as
    /// [`SynapseError::GraphqlErrors`] and are **distinct** from transport
    /// failures ([`SynapseError::Network`] / [`SynapseError::Http`]).
    ///
    /// # Errors
    /// - [`SynapseError::GraphqlErrors`] – server returned HTTP 200 with an
    ///   `errors` array; inspect the inner `Vec` for details.
    /// - [`SynapseError::Http`] – server returned a 4xx or 5xx status.
    /// - [`SynapseError::Network`] – a transport-level failure occurred.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use synapse_sdk::{SynapseClient, SynapseError};
    /// use serde_json::json;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let client = SynapseClient::new("https://api.example.com", "your-api-key");
    ///
    /// let query = r#"{ transactions { id status } }"#;
    ///
    /// match client.graphql().query(query, None).await {
    ///     Ok(data) => println!("{}", data),
    ///     Err(SynapseError::GraphqlErrors(errs)) => {
    ///         eprintln!("GraphQL errors: {:?}", errs);
    ///     }
    ///     Err(e) => eprintln!("transport error: {}", e),
    /// }
    /// # }
    /// ```
    pub async fn query(&self, query: &str, variables: Option<Value>) -> Result<Value, SynapseError> {
        let req = GraphqlRequest { query, variables };
        let envelope: GraphqlEnvelope = self.client.post("/graphql", &req).await?;

        if let Some(errors) = envelope.errors {
            if !errors.is_empty() {
                return Err(SynapseError::GraphqlErrors(errors));
            }
        }

        Ok(envelope.data.unwrap_or(Value::Null))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::SynapseClient;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn query_returns_data_on_happy_path() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .and(header("X-API-Key", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": { "transactions": [{ "id": "abc", "status": "pending" }] }
            })))
            .mount(&server)
            .await;

        let client = SynapseClient::new(server.uri(), "test-key");
        let result = client
            .graphql()
            .query("{ transactions { id status } }", None)
            .await;

        assert!(result.is_ok(), "expected Ok, got: {:?}", result);
        let data = result.unwrap();
        assert_eq!(data["transactions"][0]["id"], "abc");
        assert_eq!(data["transactions"][0]["status"], "pending");
    }

    #[tokio::test]
    async fn query_surfaces_graphql_errors_distinctly_from_transport_errors() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "errors": [{ "message": "Unknown field `foo`", "locations": [] }]
            })))
            .mount(&server)
            .await;

        let client = SynapseClient::new(server.uri(), "test-key");
        let result = client.graphql().query("{ foo }", None).await;

        assert!(
            matches!(result, Err(SynapseError::GraphqlErrors(_))),
            "expected GraphqlErrors, got: {:?}",
            result
        );
        if let Err(SynapseError::GraphqlErrors(errs)) = result {
            assert!(!errs.is_empty());
            assert_eq!(errs[0]["message"], "Unknown field `foo`");
        }
    }

    #[tokio::test]
    async fn query_sends_variables_when_provided() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .and(header("X-API-Key", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": { "transaction": { "id": "xyz", "status": "completed" } }
            })))
            .mount(&server)
            .await;

        let client = SynapseClient::new(server.uri(), "test-key");
        let vars = serde_json::json!({ "id": "xyz" });
        let result = client
            .graphql()
            .query(
                "query GetTx($id: ID!) { transaction(id: $id) { id status } }",
                Some(vars),
            )
            .await;

        assert!(result.is_ok(), "expected Ok, got: {:?}", result);
        let data = result.unwrap();
        assert_eq!(data["transaction"]["id"], "xyz");
    }

    #[tokio::test]
    async fn query_returns_http_error_on_4xx() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
            .mount(&server)
            .await;

        let client = SynapseClient::new(server.uri(), "bad-key");
        let result = client
            .graphql()
            .query("{ transactions { id } }", None)
            .await;

        assert!(
            matches!(result, Err(SynapseError::Api { status: 401, .. })),
            "expected Api 401, got: {:?}",
            result
        );
    }
}
