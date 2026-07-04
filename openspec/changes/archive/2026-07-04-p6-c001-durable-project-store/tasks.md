## 1. New adapter crate

- [x] 1.1 Create `crates/fpa-store-pg` (add to workspace `members`); deps: `fpa-ports`, `fpa-domain`, `tokio-postgres` 0.7.18 (`with-serde_json-1`), `deadpool-postgres` 0.14.1, `serde_json`, `async-trait`, `thiserror`. Workspace lints. NOT depended on by `fpa-app`.
- [x] 1.2 Bundle `schema.sql`: `CREATE TABLE IF NOT EXISTS fpa_projects (id uuid primary key, name text not null, schema_version integer not null, body jsonb not null, updated_at timestamptz not null default now())`.

## 2. PgProjectStore

- [x] 2.1 `PgProjectStore { pool: deadpool_postgres::Pool }`; `connect(db_url) -> Result<Self, PortError>` builds the pool and runs `schema.sql` (idempotent) at init.
- [x] 2.2 Impl `ProjectStore::put`: `INSERT INTO fpa_projects (id,name,schema_version,body) VALUES ($1,$2,$3,$4) ON CONFLICT (id) DO UPDATE SET name=EXCLUDED.name, schema_version=EXCLUDED.schema_version, body=EXCLUDED.body, updated_at=now()`. `body` = `serde_json::to_value(project)`.
- [x] 2.3 Impl `ProjectStore::get`: `SELECT body FROM fpa_projects WHERE id=$1`; deserialize to `Project`; no row → `Ok(None)`.
- [x] 2.4 Map pool/query errors → `PortError::Transport` (connect/pool) / `PortError::Downstream` (query) / `PortError::Decode` (serde). No `unwrap`/`expect` in lib.

## 3. Config + composition root

- [x] 3.1 `fpa-gateway::config`: add `FPA_PROJECT_DB_URL` (optional `project_db_url: Option<String>`); redact it in the manual `Debug`.
- [x] 3.2 `fpa-gateway::state::AppState::new`: if `project_db_url` is set, build `PgProjectStore::connect(...)` (fall back to in-mem on connect error with a `tracing::warn!`, or fail startup — prefer explicit: log + in-mem fallback so a bad URL doesn't hard-crash the agent); else `InMemoryProjectStore`.
- [x] 3.3 `fpa-gateway` gains a dep on `fpa-store-pg` (composition root only).

## 4. Verification

- [x] 4.1 `cargo check/clippy/fmt` green (batched at section boundary).
- [x] 4.2 Unit: `Project` → `serde_json::Value` → `Project` round-trip is lossless (no DB).
- [x] 4.3 `#[ignore]` integration test with `testcontainers` Postgres: `put` → drop pool → new `PgProjectStore::connect` → `get` returns the same aggregate (restart-survival). Document `cargo test -p fpa-store-pg -- --ignored`.
- [x] 4.4 Note in the change/reflection: the `#[ignore]`d test needs Docker (unavailable this session); it is NOT run here — no silent green.
