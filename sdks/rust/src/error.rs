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

    /// A network-level failure occurred before a response was received.
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    /// The server returned a non-success status with a JSON error message.
    #[error("API error {status}: {message}")]
    Api { status: u16, message: String },

    /// The requested resource was not found (HTTP 404).
    #[error("not found: {0}")]
    NotFound(String),

    /// A pagination cursor is malformed or has expired (HTTP 400).
    #[error("invalid cursor: {0}")]
    InvalidCursor(String),

    /// The server returned HTTP 200 but the GraphQL response contained an
    /// `errors` array. These are distinct from transport/network errors.
    #[error("GraphQL errors: {0:?}")]
    GraphqlErrors(Vec<serde_json::Value>),
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
            SynapseError::NotFound(_)
            | SynapseError::InvalidCursor(_)
            | SynapseError::GraphqlErrors(_) => false,
        }
    }
}
