## Why

The artifact **write** path is store-backed (p5–p7), but the **read** path never
caught up: `project.inspect` and `project.list` are still catalogued
`TargetPort::Forge` and route through `dispatch_forge` to `forge.describe_table` /
`forge.list_tables` — querying **forge** (which has no projects table, and which
`project.inspect` calls with `"<unspecified>"`, ignoring the `project_id`) for data
that lives in the agent's `ProjectStore`. This change makes the store a complete CRUD
surface by moving the reads to it.

## What Changes

- **`project.inspect` → store read:** retarget the catalog entry to
  `TargetPort::Store`; remove `project.inspect` from `dispatch_forge`'s shared arm;
  add a `dispatch_store` arm that parses `project_id` and returns `store.get` (the
  whole aggregate). Unknown id → clean `Downstream("unknown project …")`; no forge
  call.
- **`ProjectStore::list()` (new port method):** add
  `list() -> Result<Vec<Project>, PortError>` to the `fpa-ports` trait.
  - `InMemoryProjectStore`: `read().await.values().cloned().collect()`.
  - `PgProjectStore`: `SELECT body FROM fpa_projects` via `client.query`, each row's
    `body` deserialized through `serde_json::from_value` (mirrors `get`).
- **`project.list` → store list:** retarget to `TargetPort::Store`; remove from
  `dispatch_forge`; add a `dispatch_store` arm → `store.list()` → `{"projects":[…]}`.
  Result sorted by id for deterministic output (Base Rule 35).
- `forge.table.describe` / `forge.table.list` stay `TargetPort::Forge` (genuine forge
  reads) — only the `project.*` kinds move.

## Capabilities

### New Capabilities
- `project-store-reads`: `project.inspect` and `project.list` read from the agent-owned `ProjectStore` (get + a new `list()` port method), completing the artifact CRUD read side; no forge call for project reads.

### Modified Capabilities

## Impact

- `fpa-ports` (new `ProjectStore::list`), `fpa-app` (`InMemoryProjectStore::list`;
  `catalog.rs` retargets; `task_runner.rs` `dispatch_store` inspect/list arms + remove
  from `dispatch_forge`), `fpa-store-pg` (`PgProjectStore::list`). No new deps.

## Open Questions
- **RESOLVED (assess leans):** whole-aggregate inspect; full list now (no pagination);
  sort-by-id for determinism; single-tenant list (per-operator RLS is future debt —
  gate/forge remain the authz authority); Pg `list` proven via the `#[ignore]`d
  container test (Docker unavailable this session).
