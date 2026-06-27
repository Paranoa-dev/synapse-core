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

    /// The server returned a not-found result for a resource lookup.
    #[error("not found: {0}")]
    NotFound(String),

    /// The server rejected a cursor or pagination token.
    #[error("invalid cursor: {0}")]
    InvalidCursor(String),

    /// A network-level failure occurred before a response was received.
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    /// The body could not be decoded from the server response.
    #[error("decode error: {0}")]
    Decode(#[from] serde_json::Error),
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
            SynapseError::Decode(_) => false,
        }
    }
}
