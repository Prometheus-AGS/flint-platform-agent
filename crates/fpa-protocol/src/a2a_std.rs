//! A2A v1.0 standard types, re-exported behind the `fpa-protocol` boundary.
//!
//! We adopt `a2a-protocol-types` (serde-only, A2A v1.0) but **wrap it here** so
//! the rest of the workspace depends on `fpa-protocol`, never on the external
//! crate directly. If the upstream crate is swapped or pinned differently, only
//! this module changes (see `library-candidates.json` / decision-log).

/// A2A artifact.
pub use a2a_protocol_types::Artifact as A2aArtifact;
/// Canonical A2A task object.
pub use a2a_protocol_types::Task as A2aTask;
/// Canonical A2A task lifecycle state (`TASK_STATE_*`).
pub use a2a_protocol_types::TaskState as A2aTaskState;
/// Agent card / skill advertisement.
pub use a2a_protocol_types::{AgentCard, AgentSkill};
/// A2A message + role.
pub use a2a_protocol_types::{Message as A2aMessage, MessageRole as A2aMessageRole};

use crate::a2a::TaskEvent;
use fpa_domain::TaskId;

/// Project a standard A2A [`A2aTaskState`] onto this agent's outward
/// [`TaskEvent`] for the given task.
#[must_use]
pub fn task_event_from_state(task_id: TaskId, state: &A2aTaskState) -> TaskEvent {
    match state {
        A2aTaskState::Submitted => TaskEvent::Submitted { task_id },
        A2aTaskState::Working => TaskEvent::StatusUpdate {
            task_id,
            message: "working".to_owned(),
        },
        A2aTaskState::InputRequired | A2aTaskState::AuthRequired => TaskEvent::InputRequired {
            task_id,
            prompt: "additional input required".to_owned(),
        },
        A2aTaskState::Completed => TaskEvent::Completed {
            task_id,
            result: serde_json::Value::Null,
        },
        A2aTaskState::Failed | A2aTaskState::Rejected => TaskEvent::Failed {
            task_id,
            reason: "task did not complete".to_owned(),
        },
        A2aTaskState::Canceled => TaskEvent::Failed {
            task_id,
            reason: "canceled".to_owned(),
        },
        // `Unspecified` (proto 0-value) or any future state → opaque status.
        _ => TaskEvent::StatusUpdate {
            task_id,
            message: "unspecified".to_owned(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn maps_completed_state() {
        let id = TaskId(Uuid::nil());
        let ev = task_event_from_state(id, &A2aTaskState::Completed);
        assert!(matches!(ev, TaskEvent::Completed { .. }));
    }

    #[test]
    fn maps_failed_and_rejected_to_failed() {
        let id = TaskId(Uuid::nil());
        assert!(matches!(
            task_event_from_state(id, &A2aTaskState::Rejected),
            TaskEvent::Failed { .. }
        ));
    }
}
