## Why

The agent has never run **as a container against a live endpoint** — every proof so
far drove the router in-process (p5 mock-boundary) or ran unit tests. To smoke-test
the real binary over HTTP we need it built into an image and stood up with its
dependencies. The agent's config **requires** `FPA_FORGE_URL` / `FPA_FABRIC_ENDPOINT`
/ `FPA_GATE_ADMIN_URL` at startup (or it aborts before binding), so a dependency
must exist for it to boot. Per the operator decision, dependencies are **HTTP stubs**
(the probe surface is just 3 static GETs), not the real siblings (a future phase).

## What Changes

- **`smoke/Dockerfile`** — multi-stage: `rust:1.93-slim-bookworm` builder →
  `cargo build -p fpa-gateway --release --bin fpa-gateway` → `debian:bookworm-slim`
  runtime copying the single binary; runs it (binds `0.0.0.0:8088`).
- **`smoke/stubs/`** — wiremock mappings answering the three adapter probes:
  `GET /openapi.json` (minimal OpenAPI doc), `GET /healthz` → 200, `GET /routes` →
  `{"routes":[]}`.
- **`smoke/compose.smoke.yml`** — services:
  - `postgres` (`postgres:17-alpine`) — the agent's `FPA_PROJECT_DB_URL` target.
  - `deps` (`wiremock/wiremock`) — mounts `stubs/`, serves the 3 GETs.
  - `agent` (built from `smoke/Dockerfile`) — env: `FPA_FORGE_URL`/`FABRIC`/`GATE`
    → the `deps` stub; `FPA_PROJECT_DB_URL` → `postgres`; `FPA_GATE_JWT_KEY` (HS256
    shared secret for the smoke's minted bearer); `FPA_GATEWAY_ADDR=0.0.0.0:8088`;
    **publishes `8088:8088`**; `depends_on` postgres + deps (healthchecks).
- **`smoke/README.md`** — how to bring it up/down and what each service is.

## Capabilities

### New Capabilities
- `agent-container-compose`: A Dockerfile that builds the `fpa-gateway` binary and a compose stack (agent + postgres + wiremock stub) that stands the agent up on `:8088` with all required config satisfied — the runnable target for the live smoke.

### Modified Capabilities

## Impact

- New `smoke/` files (Dockerfile, compose.smoke.yml, stubs/, README). No Rust change —
  builds the existing `fpa-gateway` binary. Stock images only.

## Open Questions
- **RESOLVED (operator):** deps = wiremock stubs; real siblings are a future phase.
- Minimal `/openapi.json` body: enough for `forge.table.list`/`describe` to not error
  in the smoke — the smoke exercises the **store** for project data, so an empty-ish
  `{"components":{"schemas":{}}}` suffices (decide exact body in tasks).
