//! Shared error type returned by every port.
//!
//! Adapters map their transport-specific failures into these variants so the app
//! layer handles a single error surface.

use thiserror::Error;

/// An error from a downstream plane reached through a port.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum PortError {
    /// The downstream plane was unreachable or returned a transport error.
    #[error("transport error: {0}")]
    Transport(String),

    /// The operator's identity/claims were rejected by the downstream plane.
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    /// The downstream plane returned an application-level error.
    #[error("downstream error: {0}")]
    Downstream(String),

    /// A response could not be decoded.
    #[error("decode error: {0}")]
    Decode(String),
}
