//! A2A (Agent2Agent) task lifecycle events.
//!
//! These project the domain [`fpa_domain::TaskState`] onto the wire as a
//! forward-compatible tagged enum.

use fpa_domain::TaskId;
use serde::{Deserialize, Serialize};

/// An event describing a transition or update for an A2A administrative task.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TaskEvent {
    /// The task was accepted and assigned an id.
    Submitted { task_id: TaskId },

    /// Progress update while the task executes.
    StatusUpdate { task_id: TaskId, message: String },

    /// The task needs further operator input to proceed.
    InputRequired { task_id: TaskId, prompt: String },

    /// The task produced an artifact (structured result payload).
    Artifact {
        task_id: TaskId,
        artifact: serde_json::Value,
    },

    /// Terminal success.
    Completed {
        task_id: TaskId,
        result: serde_json::Value,
    },

    /// Terminal failure.
    Failed { task_id: TaskId, reason: String },

    /// Unrecognized or future variant — preserved without loss.
    #[serde(other)]
    Unknown,
}
