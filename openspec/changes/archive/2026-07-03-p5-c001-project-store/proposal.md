## Why

The `Project` aggregate (`fpa-domain::project::Project`) is the hub artifact this
agent administers, but it has **no persistence target**. Forge has no `projects`
table and its REST insert is flat-row (`POST /{schema}/{table}`) — a poor fit for a
nested aggregate (applications, sub-agents, schemas, realtime, entity-meta). Today
`project.create` routes to `forge.create_entity("Projects", …)`, which would 404
(no such table). **Operator decision (analyze):** the Project persists in an
**agent-owned `ProjectStore` port** (in-memory now, durable later), mirroring the
existing `TaskStore` precedent and unblocking the integration proof with zero
cross-repo dependency.

## What Changes

- Add a `ProjectStore` **port** to `fpa-ports`: `put(&Project)` / `get(&ProjectId)`,
  `Send + Sync`, `PortError` on failure.
- Add an in-memory adapter (`InMemoryProjectStore`, `RwLock<HashMap<ProjectId,
  Project>>`) — following `fpa-app::TaskStore`. Durable backend is a later phase.
- Rewire `task_runner`'s `project.create` to build a `Project` from the validated
  task input and call `ProjectStore.put`, returning the stored artifact — **not**
  `forge.create_entity`.
- Inject the store at the composition root (`fpa-gateway::state::AppState` +
  `TaskRunner`).

## Capabilities

### New Capabilities
- `project-store`: Agent-owned persistence of the nested `Project` aggregate behind a port, with an in-memory adapter and `project.create` wired to it.

### Modified Capabilities

## Impact

- `fpa-ports` (new `ProjectStore` trait), `fpa-app` (in-mem adapter + `TaskRunner`
  holds the store + `dispatch_forge`/dispatch `project.create` rewire), `fpa-gateway`
  (compose the store into `AppState`).
- No new dependencies.

## Open Questions
- **RESOLVED (operator):** agent-owned store, not forge. Durable backend deferred.
