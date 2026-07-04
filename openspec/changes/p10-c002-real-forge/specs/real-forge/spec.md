## ADDED Requirements

### Requirement: The real forge Postgres image runs with extensions + migrations

The smoke stack SHALL run forge's **real** Postgres image (PG18 + pgvector +
pg_graphql, from `../flint-forge/docker/postgres/Dockerfile`) and apply forge's real
migrations (`sqlx migrate run --source migrations`) before the gateway serves.
`pg_graphql` MUST be created (a migration issues `CREATE EXTENSION`).

#### Scenario: Forge PG up with schema + pg_graphql

- **WHEN** the forge Postgres starts and the migrate step completes
- **THEN** the forge schema exists and `pg_graphql` is installed (the gateway's `/graphql` + reflected `/openapi.json` work)

### Requirement: The real Quarry gateway is containerized and served

A `fdb-gateway` container image (authored — none exists in forge) SHALL build the real
Quarry gateway crate and run it against the real forge Postgres via `DATABASE_URL`,
serving `/openapi.json`, `/graphql`, and the reflected `POST /{schema}/{table}`. The
agent's `FPA_FORGE_URL` MUST point at it.

#### Scenario: Agent forge reads hit the real gateway

- **WHEN** the agent calls `GET /openapi.json` (or a store-backed op that reads forge metadata)
- **THEN** the real `fdb-gateway` responds from the migrated schema — no stub

#### Scenario: Gateway image builds without a live DB

- **WHEN** `fdb-gateway.Dockerfile` is built
- **THEN** it compiles the gateway (runtime sqlx — no database needed at build time) and produces a runnable image
