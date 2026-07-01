## 1. Task store

- [ ] 1.1 Define `TaskRecord` (id, kind, state, timestamps) and a `TaskStore` over `tokio::sync::RwLock<HashMap<TaskId, TaskRecord>>`.
- [ ] 1.2 Add the store to `AppState` (shared).

## 2. Wire A2A endpoints

- [ ] 2.1 `submit`: insert a record with the run's terminal state.
- [ ] 2.2 `GET /a2a/tasks/{id}`: return the recorded `TaskEvent`; unknown id → 404 via `ApiError::not_found`.
- [ ] 2.3 cancel: transition non-terminal → canceled; terminal → conflict/again explicit; unknown → 404.

## 3. Verification

- [ ] 3.1 `cargo check/clippy/fmt` green.
- [ ] 3.2 Test: submit then status returns the recorded state.
- [ ] 3.3 Test: status of unknown id → 404.
- [ ] 3.4 Test: cancel non-terminal → canceled; cancel terminal → not a false success.
