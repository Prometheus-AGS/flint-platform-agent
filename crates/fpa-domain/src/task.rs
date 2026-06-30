//! Administrative task domain model (the A2A unit of work).

use crate::ids::{OperatorId, TaskId};
use serde::{Deserialize, Serialize};

/// Lifecycle state of an [`AdminTask`].
///
/// `#[non_exhaustive]` so new states can be added without breaking matchers.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    /// Submitted, not yet started.
    Submitted,
    /// Currently executing against the fabric.
    Working,
    /// Awaiting operator input before it can continue.
    InputRequired,
    /// Finished successfully.
    Completed,
    /// Failed; carries a human-readable reason.
    Failed { reason: String },
    /// Cancelled by the operator.
    Canceled,
}

/// An administrative task an operator (or another agent) runs against the fabric.
///
/// The `kind` and `input` are intentionally open (`String` + `serde_json::Value`)
/// at the domain layer; concrete task catalogs are defined by the app layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdminTask {
    pub id: TaskId,
    pub operator: OperatorId,
    /// Catalog key for the task, e.g. `"forge.table.inspect"`.
    pub kind: String,
    /// Structured task input, validated by the app layer against the catalog.
    pub input: serde_json::Value,
    pub state: TaskState,
}
