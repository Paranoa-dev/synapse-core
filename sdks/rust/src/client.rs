use crate::error::SynapseError;
use crate::retry::{retry_with_backoff, DEFAULT_BASE_DELAY_MS, DEFAULT_MAX_ATTEMPTS};
use serde::de::DeserializeOwned;
use serde::Serialize;

/// HTTP client for the Synapse public API.
///
/// Construct via [`SynapseClient::new`] for simple use, or [`SynapseClient::builder`]
/// for full control over retry settings and admin key configuration.
#[derive(Clone)]
pub struct SynapseClient {
    pub(crate) http: reqwest::Client,
    pub(crate) base_url: String,
    pub(crate) api_key: String,
    pub(crate) admin_key: Option<String>,
    pub(crate) max_attempts: u32,
    pub(crate) base_delay_ms: u64,
}

/// Builder for [`SynapseClient`].
pub struct SynapseClientBuilder {
    base_url: String,
    api_key: String,
    admin_key: Option<String>,
    max_attempts: u32,
    base_delay_ms: u64,
}

impl SynapseClient {
    /// Construct a client with default retry settings.
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        SynapseClient {
            http: reqwest::Client::new(),
            base_url: base_url.into(),
            api_key: api_key.into(),
            admin_key: None,
            max_attempts: DEFAULT_MAX_ATTEMPTS,
            base_delay_ms: DEFAULT_BASE_DELAY_MS,
        }
    }

    /// Return a builder for constructing a [`SynapseClient`].
    pub fn builder(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
    ) -> SynapseClientBuilder {
        SynapseClientBuilder {
            base_url: base_url.into(),
            api_key: api_key.into(),
            admin_key: None,
            max_attempts: DEFAULT_MAX_ATTEMPTS,
            base_delay_ms: DEFAULT_BASE_DELAY_MS,
        }
    }

    /// Issue an authenticated GET request to `path`.
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, SynapseError> {
        let url = format!("{}{}", self.base_url, path);
        let key = self.api_key.clone();
        let http = self.http.clone();
        retry_with_backoff(self.max_attempts, self.base_delay_ms, || {
            let url = url.clone();
            let key = key.clone();
            let http = http.clone();
            async move {
                let resp = http
                    .get(&url)
                    .header("X-API-Key", &key)
                    .send()
                    .await
                    .map_err(SynapseError::Network)?;
                parse_response(resp).await
            }
        })
        .await
    }

    /// Issue an authenticated GET request with query parameters.
    pub async fn get_query<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T, SynapseError> {
        let url = format!("{}{}", self.base_url, path);
        let key = self.api_key.clone();
        let http = self.http.clone();
        let query = query.to_vec();
        retry_with_backoff(self.max_attempts, self.base_delay_ms, || {
            let url = url.clone();
            let key = key.clone();
            let http = http.clone();
            let query = query.clone();
            async move {
                let resp = http
                    .get(&url)
                    .header("X-API-Key", &key)
                    .query(&query)
                    .send()
                    .await
                    .map_err(SynapseError::Network)?;
                parse_response(resp).await
            }
        })
        .await
    }

    /// Issue an admin-authenticated GET request (`X-Admin-Key` header).
    ///
    /// Returns [`SynapseError::AdminKeyNotConfigured`] if no admin key is set.
    pub async fn admin_get<T: DeserializeOwned>(&self, path: &str) -> Result<T, SynapseError> {
        let admin_key = self
            .admin_key
            .clone()
            .ok_or(SynapseError::AdminKeyNotConfigured)?;
        let url = format!("{}{}", self.base_url, path);
        let http = self.http.clone();
        retry_with_backoff(self.max_attempts, self.base_delay_ms, || {
            let url = url.clone();
            let admin_key = admin_key.clone();
            let http = http.clone();
            async move {
                let resp = http
                    .get(&url)
                    .header("X-Admin-Key", &admin_key)
                    .send()
                    .await
                    .map_err(SynapseError::Network)?;
                parse_response(resp).await
            }
        })
        .await
    }

    /// Issue an admin-authenticated PATCH request with a JSON body.
    ///
    /// Returns [`SynapseError::AdminKeyNotConfigured`] if no admin key is set.
    pub async fn admin_patch<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, SynapseError> {
        let admin_key = self
            .admin_key
            .clone()
            .ok_or(SynapseError::AdminKeyNotConfigured)?;
        let url = format!("{}{}", self.base_url, path);
        let http = self.http.clone();
        let body_json =
            serde_json::to_string(body).map_err(|e| SynapseError::Encode(e.to_string()))?;
        retry_with_backoff(self.max_attempts, self.base_delay_ms, || {
            let url = url.clone();
            let admin_key = admin_key.clone();
            let http = http.clone();
            let body_json = body_json.clone();
            async move {
                let resp = http
                    .patch(&url)
                    .header("X-Admin-Key", &admin_key)
                    .header("Content-Type", "application/json")
                    .body(body_json)
                    .send()
                    .await
                    .map_err(SynapseError::Network)?;
                parse_response(resp).await
            }
        })
        .await
    }

    /// Access admin-scoped resources.
    pub fn admin(&self) -> crate::resources::admin::Admin<'_> {
        crate::resources::admin::Admin { client: self }
    }

    /// Access transaction resources.
    pub fn transactions(&self) -> crate::resources::transactions::Transactions<'_> {
        crate::resources::transactions::Transactions { client: self }
    }
}

/// Shared response parser: surfaces non-2xx as `SynapseError::Api`.
async fn parse_response<T: DeserializeOwned>(
    resp: reqwest::Response,
) -> Result<T, SynapseError> {
    let status = resp.status().as_u16();
    if status >= 400 {
        let message = resp.text().await.unwrap_or_default();
        return Err(SynapseError::Api { status, message });
    }
    resp.json::<T>()
        .await
        .map_err(|e| SynapseError::Decode(e.to_string()))
}

impl SynapseClientBuilder {
    /// Set the admin key for admin-scoped endpoints.
    pub fn admin_key(mut self, key: impl Into<String>) -> Self {
        self.admin_key = Some(key.into());
        self
    }

    /// Set the maximum total number of attempts (default: 3).
    pub fn max_attempts(mut self, n: u32) -> Self {
        self.max_attempts = n.max(1);
        self
    }

    /// Disable retry behaviour.
    pub fn disable_retries(mut self) -> Self {
        self.max_attempts = 1;
        self
    }

    /// Set the base delay in milliseconds for exponential backoff (default: 200).
    pub fn base_delay_ms(mut self, ms: u64) -> Self {
        self.base_delay_ms = ms;
        self
    }

    /// Build the [`SynapseClient`].
    pub fn build(self) -> SynapseClient {
        SynapseClient {
            http: reqwest::Client::new(),
            base_url: self.base_url,
            api_key: self.api_key,
            admin_key: self.admin_key,
            max_attempts: self.max_attempts,
            base_delay_ms: self.base_delay_ms,
        }
    }
}
