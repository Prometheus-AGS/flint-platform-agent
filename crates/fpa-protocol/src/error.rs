//! Protocol-layer error type.

use thiserror::Error;

/// Errors raised while encoding/decoding or validating protocol payloads.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ProtocolError {
    /// A payload failed serde encoding/decoding.
    #[error("protocol serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    /// A payload was structurally valid JSON but violated a protocol invariant.
    #[error("invalid protocol payload: {0}")]
    Invalid(String),
}
