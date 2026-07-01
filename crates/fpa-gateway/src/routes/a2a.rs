//! A2A (Agent2Agent) surface — administrative task lifecycle endpoints.
//!
//! Operators and other agents submit administrative tasks here; the gateway
//! hands them to the app-layer `TaskRunner` (through ports) and reports lifecycle
//! transitions as [`fpa_protocol::TaskEvent`] frames.
//!
//! Stub: endpoints accept/return well-typed payloads but do not yet drive the
//! runner — bodies return a `submitted`/`status_update` placeholder.

use crate::{api_error::ApiError, identity::OperatorContext, state::AppState};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use fpa_app::{AppError, AuthContext};
use fpa_domain::{AdminTask, OperatorId, TaskId, TaskState};
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
    /// Catalog key, e.g. `"forge.table.list"`.
    kind: String,
    /// Structured task input, validated against the catalog.
    #[serde(default)]
    input: serde_json::Value,
}

/// `POST /a2a/tasks` — submit and run a new administrative task.
///
/// Builds an [`AdminTask`] from the body + gate-derived operator, then runs it
/// through the catalog-backed [`TaskRunner`] (validation → permission → dispatch).
async fn submit(
    State(state): State<Arc<AppState>>,
    operator: OperatorContext,
    Json(req): Json<SubmitTask>,
) -> Result<Json<TaskEvent>, ApiError> {
    let task = AdminTask {
        id: TaskId(Uuid::new_v4()),
        operator: OperatorId(Uuid::new_v4()),
        kind: req.kind,
        input: req.input,
        state: TaskState::Submitted,
    };
    let auth = AuthContext {
        subject: operator.subject.clone(),
        roles: operator.roles.clone(),
        bearer: Some(operator.bearer.clone()),
    };

    match state.runner.run(&task, &auth).await {
        Ok(result) => Ok(Json(TaskEvent::Completed {
            task_id: task.id,
            result,
        })),
        Err(AppError::UnknownTaskKind(k)) => {
            Err(ApiError::not_found(format!("unknown task kind: {k}")))
        }
        Err(AppError::InvalidInput(m)) => Err(ApiError::bad_request(m)),
        Err(AppError::Unauthorized(m)) => Err(ApiError::forbidden(m)),
        // Downstream/port failures: surface as a failed TaskEvent (202-style),
        // not a 5xx — the task was accepted but the plane is unavailable.
        Err(AppError::Port(e)) => Ok(Json(TaskEvent::Failed {
            task_id: task.id,
            reason: e.to_string(),
        })),
        // `AppError` is #[non_exhaustive]: future variants map to a generic 400
        // until handled explicitly.
        Err(other) => Err(ApiError::bad_request(other.to_string())),
    }
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
