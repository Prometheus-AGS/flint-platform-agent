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
use fpa_app::{AppError, AuthContext, CancelOutcome, TaskRecord};
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
        bearer: operator.forwardable_bearer(),
        // p5-c003 G6: carry identity provenance into the task audit.
        signature_verified: operator.signature_verified,
    };

    let kind = task.kind.clone();
    let outcome = state.runner.run(&task, &auth).await;

    // Record the terminal state so status/cancel can query it.
    let final_state = match &outcome {
        Ok(_) => TaskState::Completed,
        Err(AppError::Port(e)) => TaskState::Failed {
            reason: e.to_string(),
        },
        Err(e) => TaskState::Failed {
            reason: e.to_string(),
        },
    };
    state
        .tasks
        .record(TaskRecord {
            id: task.id,
            kind,
            state: final_state,
        })
        .await;

    match outcome {
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

/// Project a recorded [`TaskState`] onto an outward [`TaskEvent`].
fn state_to_event(task_id: TaskId, state: &TaskState) -> TaskEvent {
    match state {
        TaskState::Completed => TaskEvent::Completed {
            task_id,
            result: serde_json::Value::Null,
        },
        TaskState::Failed { reason } => TaskEvent::Failed {
            task_id,
            reason: reason.clone(),
        },
        TaskState::Canceled => TaskEvent::Failed {
            task_id,
            reason: "canceled".to_owned(),
        },
        other => TaskEvent::StatusUpdate {
            task_id,
            message: format!("{other:?}"),
        },
    }
}

/// `GET /a2a/tasks/{task_id}` — fetch current task status from the store.
///
/// Requires an authenticated operator (broken-access-control fix, p4-c002 review).
async fn status(
    State(state): State<Arc<AppState>>,
    _operator: OperatorContext,
    Path(task_id): Path<Uuid>,
) -> Result<Json<TaskEvent>, ApiError> {
    let id = TaskId(task_id);
    match state.tasks.get(id).await {
        Some(rec) => Ok(Json(state_to_event(id, &rec.state))),
        None => Err(ApiError::not_found(format!("no such task: {task_id}"))),
    }
}

/// `POST /a2a/tasks/{task_id}/cancel` — request cancellation of a task.
///
/// Requires an authenticated operator (broken-access-control fix, p4-c002 review).
async fn cancel(
    State(state): State<Arc<AppState>>,
    _operator: OperatorContext,
    Path(task_id): Path<Uuid>,
) -> Result<Json<TaskEvent>, ApiError> {
    let id = TaskId(task_id);
    match state.tasks.cancel(id).await {
        CancelOutcome::Canceled => Ok(Json(TaskEvent::Failed {
            task_id: id,
            reason: "canceled".to_owned(),
        })),
        CancelOutcome::AlreadyTerminal => {
            Err(ApiError::conflict("task already terminal; cannot cancel"))
        }
        CancelOutcome::NotFound => Err(ApiError::not_found(format!("no such task: {task_id}"))),
    }
}
