use wiremock::MockServer;

/// Spawns a mock HTTP server for testing.
///
/// Returns a running MockServer instance that can be configured with route stubs.
/// The server automatically shuts down when dropped.
pub async fn spawn_mock_server() -> MockServer {
    MockServer::start().await
}

/// Helper to stub a GET endpoint with JSON response and status code.
///
/// # Example
/// ```ignore
/// let server = spawn_mock_server().await;
/// stub_get_endpoint(
///     &server,
///     "/transactions/123",
///     200,
///     serde_json::json!({"id": "123", "status": "pending"})
/// ).await;
/// ```
pub async fn stub_get_endpoint(
    server: &MockServer,
    path: &str,
    status: u16,
    body: serde_json::Value,
) {
    use wiremock::matchers::{method, path as path_matcher};
    use wiremock::{Mock, ResponseTemplate};

    Mock::given(method("GET"))
        .and(path_matcher(path))
        .respond_with(ResponseTemplate::new(status).set_body_json(body))
        .mount(server)
        .await;
}
