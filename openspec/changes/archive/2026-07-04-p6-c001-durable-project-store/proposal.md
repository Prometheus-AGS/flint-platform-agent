## Why

The `ProjectStore` port (p5-c001) is backed only by an in-memory adapter, so
`project.create`d projects do not survive a restart (p5 debt #2). This adds a
durable, agent-owned Postgres adapter behind the **existing** port — no call-site
changes. The live 3-plane smoke that phase 6 originally paired this with is
**deferred** (operator decision at assess: Docker unavailable here, forge has no
compose); this change is the shippable half.

## What Changes

- Add a **new adapter crate `fpa-store-pg`** (workspace member) implementing
  `fpa_ports::ProjectStore` over `tokio-postgres` 0.7.18 + `deadpool-postgres`
  0.14.1. It lives **outside `fpa-app`** (hexagonal Rule 16 — infra never in the app
  layer); only the composition root (`fpa-gateway`) imports it.
- Persist the `Project` aggregate **whole** as a JSONB `body` in a single
  agent-owned table (`fpa_projects`): `put` = `INSERT … ON CONFLICT (id) DO UPDATE`,
  `get` = `SELECT body WHERE id = $1` deserialized back to `Project`.
- The adapter ensures its table exists at init (a bundled `schema.sql` /
  `CREATE TABLE IF NOT EXISTS`) — the store is agent-owned, not a forge/fabric table.
- **Composition root selects the store by config**: `AppState::new` builds a
  `PgProjectStore` when `FPA_PROJECT_DB_URL` is set, else the existing
  `InMemoryProjectStore`. The env var is optional and **redacted in `Debug`**.
- Tests: unit-level `Project` ↔ JSONB round-trip (always runs); a `testcontainers`
  0.27 Postgres **restart-survival** test (`put` → new pool → `get`) that is
  **`#[ignore]`d by default** (needs Docker, unavailable in this env).

## Capabilities

### New Capabilities
- `durable-project-store`: A Postgres-backed `ProjectStore` adapter (agent-owned, JSONB) selectable at the composition root, so projects survive a restart.

### Modified Capabilities

## Impact

- **New crate** `fpa-store-pg` (added to workspace `members`); new prod deps
  `tokio-postgres` 0.7.18 + `deadpool-postgres` 0.14.1 (verified); new dev-deps
  `testcontainers` 0.27.3 + `testcontainers-modules` 0.15.0.
- `fpa-gateway::config` (+`FPA_PROJECT_DB_URL`, redacted) and `::state` (store
  selection). No change to `fpa-app`, `fpa-ports`, or any call site.

## Open Questions
- **RESOLVED (analyze):** driver `tokio-postgres`; pool `deadpool-postgres`; agent-owned
  single JSONB table; store selected by `FPA_PROJECT_DB_URL`. TLS for a remote DB and a
  real migration tool are deferred (noted for the reflection).
