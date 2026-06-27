use crate::error::SynapseError;
use crate::retry::{retry_with_backoff, DEFAULT_BASE_DELAY_MS, DEFAULT_MAX_ATTEMPTS};
use crate::resources::admin::AdminReconciliation;
use crate::resources::transactions::Transactions;
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
    /// Create a new [`SynapseClient`] with default retry settings.
    ///
    /// This is a convenience method equivalent to `SynapseClient::builder(url, key).build()`.
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self::builder(base_url, api_key).build()
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

    /// Issue an authenticated GET request to `path` and deserialize the JSON response.
    ///
    /// The request is retried automatically according to the client's retry
    /// configuration. 4xx responses are returned immediately without retrying.
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
                let status = resp.status().as_u16();
                if status >= 400 {
                    let body = resp.text().await.unwrap_or_default();
                    return if status >= 500 {
                        Err(SynapseError::Http { status, body })
                    } else {
                        Err(SynapseError::Api {
                            status,
                            message: body,
                        })
                    };
                }
                resp.json::<T>().await.map_err(SynapseError::Network)
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
                let mut req = http.get(&url).header("X-API-Key", &key);
                for (k, v) in query.iter() {
                    req = req.query(&[(k, v)]);
                }
                let resp = req.send().await.map_err(SynapseError::Network)?;
                let status = resp.status().as_u16();
                if status >= 400 {
                    let body = resp.text().await.unwrap_or_default();
                    return if status >= 500 {
                        Err(SynapseError::Http { status, body })
                    } else {
                        Err(SynapseError::Api {
                            status,
                            message: body,
                        })
                    };
                }
                resp.json::<T>().await.map_err(SynapseError::Network)
            }
        })
        .await
    }

    /// Issue an authenticated POST request with JSON body and deserialize the JSON response.
    pub async fn post<B: serde::Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, SynapseError> {
        let url = format!("{}{}", self.base_url, path);
        let key = self.api_key.clone();
        let http = self.http.clone();
        let body_json = serde_json::to_string(body)?;
        retry_with_backoff(self.max_attempts, self.base_delay_ms, || {
            let url = url.clone();
            let key = key.clone();
            let http = http.clone();
            let body_json = body_json.clone();
            async move {
                let resp = http
                    .post(&url)
                    .header("X-API-Key", &key)
                    .header("Content-Type", "application/json")
                    .body(body_json)
                    .send()
                    .await
                    .map_err(SynapseError::Network)?;
                let status = resp.status().as_u16();
                if status >= 400 {
                    let body = resp.text().await.unwrap_or_default();
                    return if status >= 500 {
                        Err(SynapseError::Http { status, body })
                    } else {
                        Err(SynapseError::Api {
                            status,
                            message: body,
                        })
                    };
                }
                resp.json::<T>().await.map_err(SynapseError::Network)
            }
        })
        .await
    }

    /// Access transaction operations.
    pub fn transactions(&self) -> Transactions {
        Transactions::new(self)
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

// ============================================================================
// Admin API Client
// ============================================================================

/// HTTP client for the Synapse admin API.
///
/// Construct via [`AdminSynapseClient::builder`]. All requests are issued with the
/// configured admin API key and are retried automatically on transient failures.
#[derive(Clone)]
pub struct AdminSynapseClient {
    pub(crate) http: reqwest::Client,
    pub(crate) base_url: String,
    pub(crate) admin_key: String,
    pub(crate) max_attempts: u32,
    pub(crate) base_delay_ms: u64,
}

/// Builder for [`AdminSynapseClient`].
pub struct AdminSynapseClientBuilder {
    base_url: String,
    admin_key: String,
    max_attempts: u32,
    base_delay_ms: u64,
}

impl AdminSynapseClient {
    /// Return a builder for constructing an [`AdminSynapseClient`].
    pub fn builder(
        base_url: impl Into<String>,
        admin_key: impl Into<String>,
    ) -> AdminSynapseClientBuilder {
        AdminSynapseClientBuilder {
            base_url: base_url.into(),
            admin_key: admin_key.into(),
            max_attempts: DEFAULT_MAX_ATTEMPTS,
            base_delay_ms: DEFAULT_BASE_DELAY_MS,
        }
    }

    /// Issue an authenticated GET request to `path` and deserialize the JSON response.
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, SynapseError> {
        let url = format!("{}{}", self.base_url, path);
        let key = self.admin_key.clone();
        let http = self.http.clone();
        retry_with_backoff(self.max_attempts, self.base_delay_ms, || {
            let url = url.clone();
            let key = key.clone();
            let http = http.clone();
            async move {
                let resp = http
                    .get(&url)
                    .header("X-Admin-Key", &key)
                    .send()
                    .await
                    .map_err(SynapseError::Network)?;
                let status = resp.status().as_u16();
                if status >= 400 {
                    let body = resp.text().await.unwrap_or_default();
                    return if status >= 500 {
                        Err(SynapseError::Http { status, body })
                    } else {
                        Err(SynapseError::Api {
                            status,
                            message: body,
                        })
                    };
                }
                resp.json::<T>().await.map_err(SynapseError::Network)
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
        let key = self.admin_key.clone();
        let http = self.http.clone();
        let query = query.to_vec();
        retry_with_backoff(self.max_attempts, self.base_delay_ms, || {
            let url = url.clone();
            let key = key.clone();
            let http = http.clone();
            let query = query.clone();
            async move {
                let mut req = http.get(&url).header("X-Admin-Key", &key);
                for (k, v) in query.iter() {
                    req = req.query(&[(k, v)]);
                }
                let resp = req.send().await.map_err(SynapseError::Network)?;
                let status = resp.status().as_u16();
                if status >= 400 {
                    let body = resp.text().await.unwrap_or_default();
                    return if status >= 500 {
                        Err(SynapseError::Http { status, body })
                    } else {
                        Err(SynapseError::Api {
                            status,
                            message: body,
                        })
                    };
                }
                resp.json::<T>().await.map_err(SynapseError::Network)
            }
        })
        .await
    }

    /// Issue an authenticated POST request with JSON body and deserialize the JSON response.
    pub async fn post<B: serde::Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, SynapseError> {
        let url = format!("{}{}", self.base_url, path);
        let key = self.admin_key.clone();
        let http = self.http.clone();
        let body_json = serde_json::to_string(body)?;
        retry_with_backoff(self.max_attempts, self.base_delay_ms, || {
            let url = url.clone();
            let key = key.clone();
            let http = http.clone();
            let body_json = body_json.clone();
            async move {
                let resp = http
                    .post(&url)
                    .header("X-Admin-Key", &key)
                    .header("Content-Type", "application/json")
                    .body(body_json)
                    .send()
                    .await
                    .map_err(SynapseError::Network)?;
                let status = resp.status().as_u16();
                if status >= 400 {
                    let body = resp.text().await.unwrap_or_default();
                    return if status >= 500 {
                        Err(SynapseError::Http { status, body })
                    } else {
                        Err(SynapseError::Api {
                            status,
                            message: body,
                        })
                    };
                }
                resp.json::<T>().await.map_err(SynapseError::Network)
            }
        })
        .await
    }

    /// Access admin reconciliation operations.
    pub fn reconciliation(&self) -> AdminReconciliation {
        AdminReconciliation::new(self)
    }
}

impl AdminSynapseClientBuilder {
    /// Set the maximum total number of attempts, including the first (default: 3).
    pub fn max_attempts(mut self, n: u32) -> Self {
        self.max_attempts = n.max(1);
        self
    }

    /// Disable retry behaviour. The first failure is returned immediately.
    pub fn disable_retries(mut self) -> Self {
        self.max_attempts = 1;
        self
    }

    /// Set the base delay in milliseconds for exponential backoff (default: 200).
    pub fn base_delay_ms(mut self, ms: u64) -> Self {
        self.base_delay_ms = ms;
        self
    }

    /// Build the [`AdminSynapseClient`].
    pub fn build(self) -> AdminSynapseClient {
        AdminSynapseClient {
            http: reqwest::Client::new(),
            base_url: self.base_url,
            admin_key: self.admin_key,
            max_attempts: self.max_attempts,
            base_delay_ms: self.base_delay_ms,
        }
    }
}
