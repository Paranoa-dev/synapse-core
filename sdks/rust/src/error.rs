use thiserror::Error;

/// Errors returned by the Synapse SDK.
#[derive(Debug, Error)]
pub enum SynapseError {
    /// The server returned a non-success HTTP status.
    ///
    /// 5xx responses are transient (retryable). 4xx responses are permanent.
    #[error("API error {status}: {message}")]
    Api { status: u16, message: String },

    /// The requested resource was not found (HTTP 404).
    #[error("not found: {0}")]
    NotFound(String),

    /// A pagination cursor was rejected by the server (HTTP 400).
    #[error("invalid cursor: {0}")]
    InvalidCursor(String),

    /// A network-level failure occurred before a response was received.
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    /// The response body could not be decoded as the expected type.
    #[error("decode error: {0}")]
    Decode(String),

    /// An admin key is required for this endpoint but was not configured.
    #[error("admin key not configured; use SynapseClient::builder().admin_key(...)")]
    AdminKeyNotConfigured,

    /// Failed to serialize the request body.
    #[error("encode error: {0}")]
    Encode(String),

    /// Alias kept for backwards compatibility with code that checks `Http`.
    #[error("HTTP {status}: {body}")]
    Http { status: u16, body: String },
}

impl SynapseError {
    /// Returns `true` if this error may resolve on a subsequent attempt.
    pub fn is_transient(&self) -> bool {
        match self {
            SynapseError::Network(_) => true,
            SynapseError::Api { status, .. } => *status >= 500,
            SynapseError::Http { status, .. } => *status >= 500,
            _ => false,
        }
    }
}
