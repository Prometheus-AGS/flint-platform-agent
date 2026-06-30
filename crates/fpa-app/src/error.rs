//! Application-layer error type.

use fpa_ports::PortError;
use thiserror::Error;

/// Errors raised while executing a use-case.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum AppError {
    /// A downstream plane reached through a port failed.
    #[error(transparent)]
    Port(#[from] PortError),

    /// The requested task `kind` is not in the catalog.
    #[error("unknown task kind: {0}")]
    UnknownTaskKind(String),

    /// Task input failed validation against the catalog schema.
    #[error("invalid task input: {0}")]
    InvalidInput(String),

    /// The operator lacks the role required to run the task.
    #[error("unauthorized: {0}")]
    Unauthorized(String),
}
