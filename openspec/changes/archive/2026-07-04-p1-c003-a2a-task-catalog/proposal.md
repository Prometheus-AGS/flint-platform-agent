## Why

`TaskRunner::run` is a `todo!()` with no catalog — there is no way to map an A2A task `kind` to a fabric operation. This change defines the administrative task catalog (project + application management) and dispatches it through the ports, turning the A2A surface from a stub into real (if still adapter-stubbed) orchestration.

## What Changes

- Define an **A2A task catalog**: a typed registry of administrative task kinds (e.g. `project.create`, `project.inspect`, `project.list`, `application.deploy`, `forge.table.describe`) each with an input schema and the port(s) it dispatches to.
- Implement `TaskRunner::run`: validate `kind` + input against the catalog, dispatch to the relevant port, map results to `TaskEvent`.
- Wire the A2A routes (`p1-c001` state) to the catalog: `POST /a2a/tasks` validates and runs; `GET`/cancel reflect real task state.
- Adopt `a2a-protocol-types` (serde-only, A2A v1.0) **behind our own `fpa-protocol` wrapper** so the wire types track the standard but a swap stays cheap.
- Task execution emits an audit record (Base Rule 34) and respects gate-derived permissions (Base Rule 33).

## Capabilities

### New Capabilities
- `a2a-task-catalog`: The administrative task registry, input validation, port dispatch, `TaskEvent` projection, and per-task audit + permission checks.

### Modified Capabilities

## Impact

- `fpa-app` (catalog + `TaskRunner` impl), `fpa-protocol` (A2A wire types), `fpa-gateway` (A2A routes consume the runner).
- New dep (pending Open Question): `a2a-protocol-types = "0.6"` OR hand-roll on `frf-agentproto`.
- Depends on `p1-c001` (state) and `p1-c002` (Project model, for project/app task inputs).

## Open Questions

- **A2A types: adopt `a2a-protocol-types` 0.6.0 vs hand-roll on `frf-agentproto`.** Recommendation (analyze): adopt the serde-only types behind our wrapper for standards alignment; fall back to hand-roll if the early crate proves unstable. **Decide before implementation.**
- Catalog persistence: in-memory registry this phase, or persisted? (Recommend in-memory now; revisit when forge gateway lands.)
