# Goals — project-read-and-list

> Seeded from `project-artifact-depth/reflection.md` → "Recommended Next Phase"
> (Option A, no-infra). Phases 5–7 built the artifact **write** side (create, nested
> input, application.define, durable store); the **read** side never caught up —
> `project.inspect`/`project.list` still query **forge** (which has no projects
> table) for data that now lives in the agent's `ProjectStore`. This phase makes the
> store a complete CRUD surface. Fully testable without Docker.

## Primary goals

1. **`project.inspect` → store read.** Retarget `project.inspect` from
   `TargetPort::Forge` to `TargetPort::Store`; dispatch it to `ProjectStore.get` by
   `project_id`, returning the whole aggregate. Unknown id → clean not-found error
   (do not hit forge). (Its catalog schema already requires `project_id`.)

2. **`project.list` → store list (new port method).** Add
   `ProjectStore::list() -> Result<Vec<Project>, PortError>` to the port; implement
   it in both adapters (`InMemoryProjectStore` and `PgProjectStore` —
   `SELECT body FROM fpa_projects`). Retarget `project.list` to `TargetPort::Store`;
   dispatch to `store.list()`, returning the projects visible to the operator.

3. **Fold in the trivial p6 archival.** p6-c001/c002 now show `✓ Complete`;
   `openspec archive` them into `specs/` (housekeeping the p7 reflection flagged).

## Success criteria

- `project.inspect` returns a stored project by id (round-trips what `project.create`
  wrote); unknown id → clean error; **no forge call** (unit-tested via the fakes).
- `ProjectStore::list()` exists on the port + both adapters; `project.list` returns
  all stored projects; empty store → empty list. In-mem path unit-tested; the Pg
  `list` is exercised by the (still `#[ignore]`d) container test or a new one.
- p6 changes archived; `openspec list` clean of them.

## Open questions (for /kbd-assess → /kbd-analyze)

- **`project.list` pagination/filtering** — needed now, or is a full list fine at
  this scale? (Lean: **full list now**, add pagination when a real dataset demands
  it — YAGNI.)
- **`project.inspect` return shape** — whole aggregate vs a summary? (Lean: **whole
  aggregate** — it's the read of exactly what `project.create` persisted.)
- **RLS/visibility** — `project.list` says "visible to the operator", but the
  agent-owned store has no per-operator ownership column yet. (Lean: return all for
  now — the store is single-tenant this phase; note per-operator scoping as future
  debt, consistent with gate/forge being the real authz authority.)
- **Pg `list` test** — extend the `#[ignore]`d container test, or add a focused one?
  (Docker still unavailable — the in-mem `list` is the runnable proof this session.)

## Explicitly out of scope this phase (still deferred)

Live 3-plane smoke (infra-gated); durable-store runtime proof (Docker); Postgres TLS;
per-operator project ownership/RLS; MCP multi-server; fabric WS subscriptions;
OpenDesign; A2UI/React UI; Tauri; knowledge-base.
