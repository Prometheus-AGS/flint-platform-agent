## 1. Dockerfile (multi-stage)

- [x] 1.1 `smoke/Dockerfile`: builder `FROM rust:1.93-slim-bookworm` — copy the workspace, `cargo build -p fpa-gateway --release --bin fpa-gateway` (add `--locked`; install `pkg-config`/`libssl-dev`/`ca-certificates` if the build needs them for reqwest rustls — verify at build).
- [x] 1.2 Runtime `FROM debian:bookworm-slim` — `ca-certificates` only; copy `/target/release/fpa-gateway`; `EXPOSE 8088`; `ENTRYPOINT ["/usr/local/bin/fpa-gateway"]`.
- [x] 1.3 Add a `.dockerignore` (target/, .git/, .kbd-orchestrator/, node_modules/) so the build context is small.

## 2. Dependency stub (wiremock)

- [x] 2.1 `smoke/stubs/mappings/*.json`: `GET /openapi.json` → 200 `{"openapi":"3.1.0","components":{"schemas":{}}}`; `GET /healthz` → 200 `{"status":"ok"}`; `GET /routes` → 200 `{"routes":[]}`.

## 3. Compose stack

- [x] 3.1 `smoke/compose.smoke.yml`: `postgres` (`postgres:17-alpine`, POSTGRES_PASSWORD, healthcheck `pg_isready`); `deps` (`wiremock/wiremock`, mount `./stubs`, healthcheck on `/__admin`); `agent` (build `.`/Dockerfile).
- [x] 3.2 `agent` env: `FPA_FORGE_URL=http://deps:8080`, `FPA_FABRIC_ENDPOINT=http://deps:8080`, `FPA_GATE_ADMIN_URL=http://deps:8080`, `FPA_PROJECT_DB_URL=postgres://postgres:postgres@postgres:5432/postgres`, `FPA_GATE_JWT_KEY=<smoke-secret>`, `FPA_GATEWAY_ADDR=0.0.0.0:8088`; `ports: ["8088:8088"]`; `depends_on` postgres+deps (service_healthy).
- [x] 3.3 `smoke/README.md`: `docker compose -f smoke/compose.smoke.yml up --build` / `down -v`; what each service is; the shared HS256 secret.

## 4. Verification

- [x] 4.1 `docker compose -f smoke/compose.smoke.yml config` validates.
- [x] 4.2 `up --build` → agent boots (no missing-config abort); `curl localhost:8088/healthz` → 200. Capture the log line "fpa-gateway listening".
- [x] 4.3 `down -v` cleans up.
