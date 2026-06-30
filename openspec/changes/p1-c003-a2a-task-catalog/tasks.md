## 1. A2A types decision (Open Question — resolve first)

- [ ] 1.1 Evaluate `a2a-protocol-types` 0.6.0 API fit vs hand-roll on `frf-agentproto`; record the call in the change + KBD decision-log.
- [ ] 1.2 If adopting: add `a2a-protocol-types = "0.6"`; re-export through `fpa-protocol` behind our own type wrapper (do not leak it into `fpa-app` signatures).

## 2. Task catalog

- [ ] 2.1 `fpa-app/src/catalog/`: `TaskKind` registry mapping kind → input schema → target port(s).
- [ ] 2.2 Seed catalog entries: `project.{create,inspect,list,update}`, `application.{define,deploy}`, `forge.table.{list,describe}` (forge-backed ones may return `PortError::Downstream` until forge ships).
- [ ] 2.3 Input validation against each entry's JSON Schema; reject unknown `kind` with `AppError::UnknownTaskKind` and bad input with `AppError::InvalidInput`.

## 3. TaskRunner dispatch

- [ ] 3.1 Implement `TaskRunner::run`: look up `kind`, validate input, dispatch to port, map `Ok`/`Err` → `TaskEvent::{Completed,Failed,...}`.
- [ ] 3.2 Enforce gate-derived permissions before dispatch (deny → `TaskEvent::Failed` with reason; never bypass — Base Rule 33).
- [ ] 3.3 Emit an audit record per task (request, kind, decision, outcome — Base Rule 34); never log secrets/claims.

## 4. Wire A2A routes

- [ ] 4.1 `POST /a2a/tasks`: build `AdminTask` from body + `OperatorContext`, run via `State` runner, return real `TaskEvent`.
- [ ] 4.2 `GET /a2a/tasks/{id}` + cancel: reflect actual task state (in-memory task store this phase).

## 5. Verification

- [ ] 5.1 `cargo check/clippy/fmt` green.
- [ ] 5.2 Unit tests with `mockall` fakes for all four ports: a known `kind` dispatches to the right port; unknown `kind` → `UnknownTaskKind`; invalid input → `InvalidInput`.
- [ ] 5.3 Test: permission-denied task yields `TaskEvent::Failed` and never calls the port.
- [ ] 5.4 Integration: `POST /a2a/tasks` end-to-end through the runner with fake adapters returns the expected `TaskEvent` JSON.
