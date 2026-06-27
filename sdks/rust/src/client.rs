use crate::error::SynapseError;
use crate::retry::{retry_with_backoff, DEFAULT_BASE_DELAY_MS, DEFAULT_MAX_ATTEMPTS};
use crate::resources::{health::Health, transactions::Transactions};
use serde::de::DeserializeOwned;

/// HTTP client for the Synapse public API.
///
/// Construct via [`SynapseClient::builder`]. All requests are issued with the
/// configured API key and are retried automatically on transient failures.
#[derive(Clone)]
pub struct SynapseClient {
    pub(crate) http: reqwest::Client,
    pub(crate) base_url: String,
    pub(crate) api_key: String,
    pub(crate) max_attempts: u32,
    pub(crate) base_delay_ms: u64,
}

/// Builder for [`SynapseClient`].
pub struct SynapseClientBuilder {
    base_url: String,
    api_key: String,
    max_attempts: u32,
    base_delay_ms: u64,
}

impl SynapseClient {
    /// Create a client with the given base URL and API key.
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> SynapseClient {
        SynapseClient::builder(base_url, api_key).build()
    }

    /// Return a builder for constructing a [`SynapseClient`].
    pub fn builder(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
    ) -> SynapseClientBuilder {
        SynapseClientBuilder {
            base_url: base_url.into(),
            api_key: api_key.into(),
            max_attempts: DEFAULT_MAX_ATTEMPTS,
            base_delay_ms: DEFAULT_BASE_DELAY_MS,
        }
    }

    /// Return a handle for transaction endpoints.
    pub fn transactions(&self) -> Transactions {
        Transactions { client: self }
    }

    /// Return a handle for health endpoints.
    pub fn health(&self) -> Health {
        Health { client: self }
    }

    fn build_url(&self, path: &str, query: &[(&str, &str)]) -> String {
        if query.is_empty() {
            format!("{}{}", self.base_url, path)
        } else {
            let query = query
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            format!("{}{}?{}", self.base_url, path, query)
        }
    }

    async fn get_response(&self, path: &str) -> Result<reqwest::Response, SynapseError> {
        let url = self.build_url(path, &[]);
        let key = self.api_key.clone();
        let http = self.http.clone();
        retry_with_backoff(self.max_attempts, self.base_delay_ms, || {
            let url = url.clone();
            let key = key.clone();
            let http = http.clone();
            async move { http.get(&url).header("X-API-Key", &key).send().await.map_err(SynapseError::Network) }
        })
        .await
    }

    /// Issue an authenticated GET request to `path` and deserialize the JSON response.
    ///
    /// The request is retried automatically according to the client's retry
    /// configuration. 4xx responses are returned immediately without retrying.
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, SynapseError> {
        let resp = self.get_response(path).await?;
        let status = resp.status().as_u16();
        if status >= 400 {
            let body = resp.text().await.unwrap_or_default();
            return Err(SynapseError::Http { status, body });
        }
        resp.json::<T>().await.map_err(SynapseError::Network)
    }

    /// Issue an authenticated GET request with query parameters and deserialize JSON.
    pub async fn get_query<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T, SynapseError> {
        let url = self.build_url(path, query);
        let key = self.api_key.clone();
        let http = self.http.clone();
        retry_with_backoff(self.max_attempts, self.base_delay_ms, || {
            let url = url.clone();
            let key = key.clone();
            let http = http.clone();
            async move {
                let resp = http.get(&url).header("X-API-Key", &key).send().await.map_err(SynapseError::Network)?;
                let status = resp.status().as_u16();
                if status >= 400 {
                    let body = resp.text().await.unwrap_or_default();
                    return Err(SynapseError::Http { status, body });
                }
                resp.json::<T>().await.map_err(SynapseError::Network)
            }
        })
        .await
    }

    /// Issue an authenticated GET request and deserialize JSON even on non-2xx status.
    pub async fn get_json_with_status<T: DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<(u16, T), SynapseError> {
        let resp = self.get_response(path).await?;
        let status = resp.status().as_u16();
        let body = resp.json::<T>().await.map_err(SynapseError::Network)?;
        Ok((status, body))
    }

    /// Issue an authenticated GET request with query parameters and deserialize JSON even on non-2xx status.
    pub async fn get_query_json_with_status<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<(u16, T), SynapseError> {
        let url = self.build_url(path, query);
        let key = self.api_key.clone();
        let http = self.http.clone();
        retry_with_backoff(self.max_attempts, self.base_delay_ms, || {
            let url = url.clone();
            let key = key.clone();
            let http = http.clone();
            async move {
                let resp = http.get(&url).header("X-API-Key", &key).send().await.map_err(SynapseError::Network)?;
                let status = resp.status().as_u16();
                let body = resp.json::<T>().await.map_err(SynapseError::Network)?;
                Ok((status, body))
            }
        })
        .await
    }

    /// Issue an authenticated GET request and return raw bytes.
    pub async fn get_bytes(&self, path: &str) -> Result<Vec<u8>, SynapseError> {
        let resp = self.get_response(path).await?;
        let status = resp.status().as_u16();
        if status >= 400 {
            let body = resp.text().await.unwrap_or_default();
            return Err(SynapseError::Http { status, body });
        }
        resp.bytes().await.map(|b| b.to_vec()).map_err(SynapseError::Network)
    }

    /// Issue an authenticated GET request with query parameters and return raw bytes.
    pub async fn get_query_bytes(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<Vec<u8>, SynapseError> {
        let url = self.build_url(path, query);
        let key = self.api_key.clone();
        let http = self.http.clone();
        retry_with_backoff(self.max_attempts, self.base_delay_ms, || {
            let url = url.clone();
            let key = key.clone();
            let http = http.clone();
            async move {
                let resp = http.get(&url).header("X-API-Key", &key).send().await.map_err(SynapseError::Network)?;
                let status = resp.status().as_u16();
                if status >= 400 {
                    let body = resp.text().await.unwrap_or_default();
                    return Err(SynapseError::Http { status, body });
                }
                resp.bytes().await.map(|b| b.to_vec()).map_err(SynapseError::Network)
            }
        })
        .await
    }
}

impl SynapseClientBuilder {
    /// Set the maximum total number of attempts, including the first (default: 3).
    ///
    /// Values below 1 are treated as 1 (no retries).
    pub fn max_attempts(mut self, n: u32) -> Self {
        self.max_attempts = n.max(1);
        self
    }

    /// Disable retry behaviour. The first failure is returned immediately.
    ///
    /// Use this when the caller manages its own retry loop.
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
            max_attempts: self.max_attempts,
            base_delay_ms: self.base_delay_ms,
        }
    }
}
