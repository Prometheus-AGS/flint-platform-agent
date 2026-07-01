//! In-memory task store backing A2A `status` / `cancel`.
//!
//! Single-process, in-memory this phase (a durable/persisted store is a later
//! concern — YAGNI). Keyed by [`TaskId`], guarded by an async `RwLock`.

use fpa_domain::{TaskId, TaskState};
use std::collections::HashMap;
use tokio::sync::RwLock;

/// A recorded task and its current lifecycle state.
#[derive(Debug, Clone)]
pub struct TaskRecord {
    pub id: TaskId,
    pub kind: String,
    pub state: TaskState,
}

/// Outcome of a cancel request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CancelOutcome {
    /// The task was non-terminal and is now canceled.
    Canceled,
    /// The task was already terminal and cannot be canceled.
    AlreadyTerminal,
    /// No task with that id is known.
    NotFound,
}

/// In-memory store of submitted tasks.
#[derive(Default)]
pub struct TaskStore {
    tasks: RwLock<HashMap<TaskId, TaskRecord>>,
}

impl TaskStore {
    /// Record a task with its (possibly terminal) state.
    pub async fn record(&self, record: TaskRecord) {
        self.tasks.write().await.insert(record.id, record);
    }

    /// Fetch a task's current record, if known.
    pub async fn get(&self, id: TaskId) -> Option<TaskRecord> {
        self.tasks.read().await.get(&id).cloned()
    }

    /// Attempt to cancel a task. Non-terminal → canceled; terminal → refused;
    /// unknown → not found. Never a silent success.
    pub async fn cancel(&self, id: TaskId) -> CancelOutcome {
        let mut tasks = self.tasks.write().await;
        match tasks.get_mut(&id) {
            None => CancelOutcome::NotFound,
            Some(rec) if is_terminal(&rec.state) => CancelOutcome::AlreadyTerminal,
            Some(rec) => {
                rec.state = TaskState::Canceled;
                CancelOutcome::Canceled
            }
        }
    }
}

/// Whether a state is terminal (cannot transition further).
fn is_terminal(state: &TaskState) -> bool {
    matches!(
        state,
        TaskState::Completed | TaskState::Failed { .. } | TaskState::Canceled
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn rec(state: TaskState) -> TaskRecord {
        TaskRecord {
            id: TaskId(Uuid::nil()),
            kind: "project.list".into(),
            state,
        }
    }

    #[tokio::test]
    async fn record_then_get() {
        let store = TaskStore::default();
        store.record(rec(TaskState::Completed)).await;
        let got = store.get(TaskId(Uuid::nil())).await.expect("present");
        assert_eq!(got.state, TaskState::Completed);
    }

    #[tokio::test]
    async fn get_unknown_is_none() {
        let store = TaskStore::default();
        assert!(store.get(TaskId(Uuid::from_u128(9))).await.is_none());
    }

    #[tokio::test]
    async fn cancel_non_terminal_then_terminal() {
        let store = TaskStore::default();
        store.record(rec(TaskState::Working)).await;
        assert_eq!(
            store.cancel(TaskId(Uuid::nil())).await,
            CancelOutcome::Canceled
        );
        // second cancel: now terminal
        assert_eq!(
            store.cancel(TaskId(Uuid::nil())).await,
            CancelOutcome::AlreadyTerminal
        );
    }

    #[tokio::test]
    async fn cancel_unknown_is_not_found() {
        let store = TaskStore::default();
        assert_eq!(
            store.cancel(TaskId(Uuid::from_u128(7))).await,
            CancelOutcome::NotFound
        );
    }
}
