//! A2A (Agent2Agent) surface — administrative task lifecycle endpoints.
//!
//! Operators and other agents submit administrative tasks here; the gateway
//! hands them to the app-layer `TaskRunner` (through ports) and reports lifecycle
//! transitions as [`fpa_protocol::TaskEvent`] frames.
//!
//! Stub: endpoints accept/return well-typed payloads but do not yet drive the
//! runner — bodies return a `submitted`/`status_update` placeholder.

use crate::{identity::OperatorContext, state::AppState};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use fpa_domain::TaskId;
use fpa_protocol::TaskEvent;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

/// Routes for the A2A surface.
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/a2a/tasks", post(submit))
        .route("/a2a/tasks/{task_id}", get(status))
        .route("/a2a/tasks/{task_id}/cancel", post(cancel))
}

/// Request body for submitting an administrative task.
#[derive(Debug, Deserialize)]
struct SubmitTask {
    /// Catalog key, e.g. `"forge.table.inspect"`.
    #[allow(dead_code)]
    kind: String,
    /// Structured task input, validated against the catalog.
    #[allow(dead_code)]
    input: serde_json::Value,
}

/// `POST /a2a/tasks` — submit a new administrative task.
///
/// The composition root now provides shared state and a gate-derived
/// [`OperatorContext`]. Catalog validation + dispatch through the runner land in
/// `p1-c003`; for now the handler proves identity + state flow end-to-end.
async fn submit(
    State(state): State<Arc<AppState>>,
    operator: OperatorContext,
    Json(_req): Json<SubmitTask>,
) -> Json<TaskEvent> {
    // Touch shared state so the wiring is exercised (real dispatch in p1-c003).
    let _runner = &state.runner;
    tracing::info!(operator = %operator.subject, "a2a task submitted");
    let task_id = TaskId(Uuid::nil());
    Json(TaskEvent::Submitted { task_id })
}

/// `GET /a2a/tasks/{task_id}` — fetch current task status.
async fn status(Path(task_id): Path<Uuid>) -> Json<TaskEvent> {
    Json(TaskEvent::StatusUpdate {
        task_id: TaskId(task_id),
        message: "task status lookup not yet implemented".to_owned(),
    })
}

/// `POST /a2a/tasks/{task_id}/cancel` — request cancellation of a task.
async fn cancel(Path(task_id): Path<Uuid>) -> Json<TaskEvent> {
    Json(TaskEvent::StatusUpdate {
        task_id: TaskId(task_id),
        message: "task cancellation not yet implemented".to_owned(),
    })
}
