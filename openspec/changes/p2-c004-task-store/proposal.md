## Why

A2A `GET /a2a/tasks/{id}` and cancel return placeholders — there is no task store,
so submitted tasks have no queryable state. This change adds an in-memory task
store so `status`/`cancel` reflect real state.

## What Changes

- Add an in-memory task store (`tokio::sync::RwLock<HashMap<TaskId, TaskRecord>>`) to the app layer / `AppState`.
- `submit` records the task + its terminal outcome.
- `GET /a2a/tasks/{id}` returns the recorded state; unknown id → 404.
- cancel transitions a non-terminal task to canceled; terminal/unknown handled explicitly.
- Single-process, in-memory this phase (durable/persisted store is a later concern — YAGNI).

## Capabilities

### New Capabilities
- `task-store`: In-memory record of submitted tasks and their state, backing A2A `status` and `cancel`.

### Modified Capabilities

## Impact

- `fpa-app` (task store type), `fpa-gateway` (A2A `status`/`cancel` read/mutate the store; `submit` records).
- No new dependencies (`tokio` present).
- Independent of the forge changes.
