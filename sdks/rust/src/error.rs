use thiserror::Error;

/// Errors returned by the Synapse SDK.
#[derive(Debug, Error)]
pub enum SynapseError {
    /// The server returned an HTTP error status.
    ///
    /// 5xx responses are transient (retryable). 4xx responses are permanent
    /// caller mistakes and are never retried.
    #[error("HTTP {status}: {body}")]
    Http { status: u16, body: String },

    /// The server returned a non-2xx response for an API request.
    #[error("API error {status}: {message}")]
    Api { status: u16, message: String },

    /// The requested resource was not found (HTTP 404).
    #[error("{0}")]
    NotFound(String),

    /// The pagination cursor is invalid or expired (HTTP 400 with "cursor").
    #[error("invalid cursor: {0}")]
    InvalidCursor(String),

    /// A network-level failure occurred before a response was received.
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    /// The response body could not be decoded as valid JSON.
    #[error("decode error: {0}")]
    Decode(String),
}

impl SynapseError {
    /// Returns `true` if this error may resolve on a subsequent attempt.
    ///
    /// Network errors and 5xx HTTP responses are transient. 4xx responses are
    /// permanent (they represent a caller mistake) and must not be retried.
    pub fn is_transient(&self) -> bool {
        match self {
            SynapseError::Network(_) => true,
            SynapseError::Http { status, .. } => *status >= 500,
            SynapseError::Api { status, .. } => *status >= 500,
            _ => false,
        }
    }
}
