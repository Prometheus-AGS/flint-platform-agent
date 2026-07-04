## 1. Real forge Postgres + migrations

- [ ] 1.1 `forge-postgres` service in `compose.real.yml`: `build: { context: ../../flint-forge, dockerfile: docker/postgres/Dockerfile }`; POSTGRES_PASSWORD; healthcheck `pg_isready`.
- [ ] 1.2 `forge-migrate` one-shot: run `sqlx migrate run --source migrations` against forge-postgres (a `sqlx-cli` container OR a tiny image; `DATABASE_URL` → forge-postgres). `depends_on: forge-postgres (service_healthy)`. Runs to completion, then exits 0.

## 2. Author the fdb-gateway Dockerfile

- [ ] 2.1 `smoke/fdb-gateway.Dockerfile`: multi-stage `rust:1.96-slim-bookworm` builder (workspace = `../flint-forge`); `cargo build -p fdb-gateway --release --bin fdb-gateway --locked`; install `pkg-config ca-certificates git libssl-dev`; neutralize `RUSTC_WRAPPER`/`RUSTFLAGS`; a `.dockerignore` excludes `.cargo/`/target/. Runtime `debian:bookworm-slim` + `ca-certificates libssl3`; copy the binary; `EXPOSE <port>`.
- [ ] 2.2 Confirm the bind port from `../flint-forge/crates/fdb-gateway/src/main.rs` (axum::serve listener); wire `FPA_FORGE_URL=http://forge-gateway:<port>`.

## 3. Wire it into compose

- [ ] 3.1 `forge-gateway` service: `build` the authored Dockerfile; env `DATABASE_URL` → forge-postgres; `depends_on: forge-migrate (service_completed_successfully)`.
- [ ] 3.2 Agent env `FPA_FORGE_URL` → the forge-gateway.

## 4. Verification

- [ ] 4.1 forge-postgres image builds (pg_graphql `.deb` + pgvector source); migrate one-shot completes.
- [ ] 4.2 `fdb-gateway.Dockerfile` builds (no DB needed); gateway serves `GET /openapi.json` → 200 with the reflected schema.
- [ ] 4.3 The agent's forge-read hop hits the real gateway.
