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

## Open Questions
- **`fdb-gateway` bind port** — confirm from `../flint-forge/crates/fdb-gateway/src/main.rs`
  at execute (was seen ~8080). Wire `FPA_FORGE_URL` to it.
- **Migrate runner:** a `sqlx-cli` one-shot container (`ghcr.io/.../sqlx` or build it) vs
  running `sqlx migrate run` inside the gateway image at startup. Decide at execute
  (lean: a dedicated one-shot so the gateway image stays minimal).
- Author the gateway Dockerfile in our `smoke/` (builds from the `../flint-forge`
  context) — does NOT edit the forge repo.
