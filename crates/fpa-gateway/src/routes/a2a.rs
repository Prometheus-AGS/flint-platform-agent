//! A2A (Agent2Agent) surface — administrative task lifecycle endpoints.
//!
//! Operators and other agents submit administrative tasks here; the gateway
//! hands them to the app-layer `TaskRunner` (through ports) and reports lifecycle
//! transitions as [`fpa_protocol::TaskEvent`] frames.
//!
//! Stub: endpoints accept/return well-typed payloads but do not yet drive the
//! runner — bodies return a `submitted`/`status_update` placeholder.

use axum::{
    Json, Router,
    extract::Path,
    routing::{get, post},
};
use fpa_domain::TaskId;
use fpa_protocol::TaskEvent;
use serde::Deserialize;
use uuid::Uuid;

/// Routes for the A2A surface.
pub fn router() -> Router {
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
/// Stub: mints a task id and acknowledges. The real handler validates `kind`
/// against the catalog and dispatches to `fpa_app::TaskRunner`.
async fn submit(Json(_req): Json<SubmitTask>) -> Json<TaskEvent> {
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
