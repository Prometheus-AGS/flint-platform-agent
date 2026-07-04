## Why

Operator directive: **nothing but REAL** — the agent's forge surface (`GET
/openapi.json`, `POST /{schema}/{table}`, `POST /graphql`) must hit the **real Quarry
gateway** over a **real forge Postgres** with the extensions applied. The agent's reads
depend on `pg_graphql` + the migrated schema, so the forge PG must be forge's real
image (per the re-sync: the CI image installs pg_graphql from a prebuilt `.deb`).

## What Changes

- **Real forge Postgres:** build `../flint-forge/docker/postgres/Dockerfile` (PG18 +
  pgvector 0.8.0 + pg_graphql 1.5.11 via `.deb`) into `compose.real.yml`.
- **Apply forge's real migrations:** a one-shot `migrate` step runs `sqlx migrate run
  --source migrations` (forge's `ci-test.sh` mechanism) against the forge PG before the
  gateway serves — so `/openapi.json` reflects the real schema and `CREATE EXTENSION
  pg_graphql` has run.
- **Author `fdb-gateway` (Quarry) Dockerfile** — none exists in forge. Thin multi-stage
  `rust:1.96`-bookworm builder → slim runtime (the re-sync confirmed `fdb-gateway` uses
  runtime `sqlx` with no `.sqlx` offline cache, so it **builds without a live DB**).
  Apply the known container lessons: exclude `.cargo`, add `libssl-dev`. It reads
  `DATABASE_URL` at runtime and `axum::serve`s (default port from its `main.rs`).
- Wire the agent's `FPA_FORGE_URL` → the real `fdb-gateway`.

## Capabilities

### New Capabilities
- `real-forge`: The real forge Quarry gateway (`fdb-gateway`, newly containerized) runs over forge's real Postgres image (pg_graphql + pgvector) with real migrations applied; the agent's forge reads/writes hit it live.

## Impact

- New `smoke/fdb-gateway.Dockerfile` (authored — builds `../flint-forge`'s `fdb-gateway`
  crate). Forge PG + migrate + gateway services contribute to `compose.real.yml`
  (p10-c004). No agent Rust change.

## Execute outcome (2026-07-04)

Grounding against the LIVE forge reshaped this change. **What is PROVEN real:**
- forge's CI PG-18 image builds (pgvector + pg_graphql) — after fixing **two forge
  Dockerfile bugs** via smoke-side `build.args` overrides (NO forge edit):
  `PGVECTOR_REF=v0.8.4` (v0.8.0 won't compile on PG18) + `PG_GRAPHQL_REF=v1.6.1`
  (v1.5.11 has no pg18 `.deb` → 404).
- **`smoke/fdb-gateway.Dockerfile` builds** the real gateway from the forge crate via a
  read-only build context (BuildKit adjacent `.dockerignore` trims forge's 34 GB
  `target/` without writing into forge). Confirmed: forge's crate builds without a DB.
- **DB pre-seed works:** `smoke/forge-bootstrap/` seeds roles + a vendored copy of
  forge's pure-SQL `flint_meta.sql` (the prereqs forge's migrations assume). `forge-
  postgres` healthy, `forge-bootstrap` exits 0.

**What is DEFERRED to best-effort** (blocked on a forge bug we cannot fix from outside):
- The live **`fdb-gateway` boot**. It runs `sqlx::migrate!` at startup, which **rejects
  forge's duplicate migration versions** (two `0005_`, two `0006_`) →
  `_sqlx_migrations_pkey: Key (version)=(5) already exists`. The filenames are compiled
  into the gateway binary, so this is un-workaroundable from `smoke/`. The offending
  files are **untracked in-flight forge work** (p5-c005), so we did NOT rename them.
- **Filed: Know-Me-Tools/flint-forge#7** — full functional spec of all three blockers.
  When forge ships unique migration versions, our gateway boots with zero smoke changes.

## Open Questions (resolved / carried)
- ~~`fdb-gateway` bind port~~ — confirmed hard-coded `0.0.0.0:8080` in its `main.rs`.
- ~~Migrate runner~~ — resolved: the gateway self-migrates via `sqlx::migrate!`; our
  bootstrap only pre-seeds prerequisites (roles + flint_meta), does NOT run migrations.
- Gateway boot re-enables when **forge#7** (dup migration versions) lands.
