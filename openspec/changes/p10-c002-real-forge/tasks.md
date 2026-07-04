## 1. Real forge Postgres + prerequisites

- [x] 1.1 `forge-postgres` service in `smoke/compose.forge.yml`: `build` forge's CI PG image (`docker/postgres/Dockerfile`) with smoke-side `build.args` overrides `PGVECTOR_REF=v0.8.4` + `PG_GRAPHQL_REF=v1.6.1` (forge's v0.8.0/v1.5.11 pins are unbuildable on PG18 — see forge#7). `pg_isready` healthcheck. PROVEN: builds + healthy.
- [x] 1.2 `forge-bootstrap` one-shot: seeds ONLY the prereqs forge's migrations assume — roles (`00-roles.sql`) + vendored `flint_meta` (`01-flint-meta.sql`, pure SQL). Does NOT run migrations (the gateway self-migrates via `sqlx::migrate!`). `depends_on: forge-postgres (healthy)`; exits 0. PROVEN.

## 2. Author the fdb-gateway Dockerfile

- [x] 2.1 `smoke/fdb-gateway.Dockerfile`: multi-stage `rust:1.90-slim-bookworm` (matches forge's Dagger toolchain, NOT 1.96); `cargo build -p fdb-gateway --release --locked`; `libssl-dev`+`libssl3`; neutralized `RUSTC_WRAPPER`/`RUSTFLAGS`. Build context = `../flint-forge`; adjacent `fdb-gateway.Dockerfile.dockerignore` trims the 34 GB `target/` WITHOUT writing into forge. PROVEN: image builds (whole forge workspace compiles, no DB needed).
- [x] 2.2 Bind port confirmed hard-coded `0.0.0.0:8080` in `main.rs`; `FPA_FORGE_URL` wires to `:8080` (host 18080 in the standalone file).

## 3. Wire it into compose

- [x] 3.1 `forge-gateway` service builds the authored Dockerfile; `DATABASE_URL` → forge-postgres; `depends_on: forge-bootstrap (completed_successfully)`.
- [ ] 3.2 Agent env `FPA_FORGE_URL` → forge-gateway — lands in the unified `compose.real.yml` (c005), gated on the gateway booting (forge#7).

## 4. Verification

- [x] 4.1 forge-postgres image builds (pgvector 0.8.4 + pg_graphql 1.6.1 debs); bootstrap one-shot completes (roles + flint_meta seeded). PROVEN live.
- [x] 4.2 `fdb-gateway.Dockerfile` builds (no DB needed). PROVEN.
- [ ] 4.3 Gateway serves `GET /openapi.json` → **DEFERRED (best-effort)**: gateway boot panics on forge's duplicate migration versions (`sqlx::migrate!` — `_sqlx_migrations_pkey Key (version)=(5) already exists`). Un-workaroundable from smoke/ (compiled into the binary). **Filed Know-Me-Tools/flint-forge#7.** Re-enables when forge ships unique migration versions.
